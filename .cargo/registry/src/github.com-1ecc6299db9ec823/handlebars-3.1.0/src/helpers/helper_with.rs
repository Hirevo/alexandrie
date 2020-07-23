use crate::block::{BlockContext, BlockParams};
use crate::context::Context;
use crate::error::RenderError;
use crate::helpers::{HelperDef, HelperResult};
use crate::json::value::JsonTruthy;
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext, Renderable};

#[derive(Clone, Copy)]
pub struct WithHelper;

impl HelperDef for WithHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Registry<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"with\""))?;

        let not_empty = param.value().is_truthy(false);
        let template = if not_empty { h.template() } else { h.inverse() };

        if not_empty {
            let mut block = BlockContext::new();

            let new_path = param.context_path();
            if let Some(new_path) = new_path {
                *block.base_path_mut() = new_path.clone();
            }

            if let Some(block_param) = h.block_param() {
                let mut params = BlockParams::new();
                if new_path.is_some() {
                    params.add_path(block_param, Vec::with_capacity(0))?;
                } else {
                    params.add_value(block_param, param.value().clone())?;
                }

                block.set_block_params(params);
            }

            rc.push_block(block);
        }

        let result = match template {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        };

        if not_empty {
            rc.pop_block();
        }
        result
    }
}

pub static WITH_HELPER: WithHelper = WithHelper;

#[cfg(test)]
mod test {
    use crate::json::value::to_json;
    use crate::registry::Registry;

    #[derive(Serialize)]
    struct Address {
        city: String,
        country: String,
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>,
    }

    #[test]
    fn test_with() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#with addr}}{{city}}{{/with}}")
            .is_ok());
        assert!(handlebars
            .register_template_string("t1", "{{#with notfound}}hello{{else}}world{{/with}}")
            .is_ok());
        assert!(handlebars
            .register_template_string("t2", "{{#with addr/country}}{{this}}{{/with}}")
            .is_ok());

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.ok().unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t2", &person);
        assert_eq!(r2.ok().unwrap(), "China".to_string());
    }

    #[test]
    fn test_with_block_param() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#with addr as |a|}}{{a.city}}{{/with}}")
            .is_ok());
        assert!(handlebars
            .register_template_string("t1", "{{#with notfound as |c|}}hello{{else}}world{{/with}}")
            .is_ok());
        assert!(handlebars
            .register_template_string("t2", "{{#with addr/country as |t|}}{{t}}{{/with}}")
            .is_ok());

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.ok().unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.ok().unwrap(), "world".to_string());

        let r2 = handlebars.render("t2", &person);
        assert_eq!(r2.ok().unwrap(), "China".to_string());
    }

    #[test]
    fn test_with_in_each() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let addr2 = Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        };

        let person2 = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr2,
            titles: vec!["programmer".to_string(), "cartographier".to_string()],
        };

        let people = vec![person, person2];

        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#each this}}{{#with addr}}{{city}}{{/with}}{{/each}}"
            )
            .is_ok());
        assert!(handlebars
            .register_template_string(
                "t1",
                "{{#each this}}{{#with addr}}{{../age}}{{/with}}{{/each}}"
            )
            .is_ok());
        assert!(handlebars
            .register_template_string(
                "t2",
                "{{#each this}}{{#with addr}}{{@../index}}{{/with}}{{/each}}"
            )
            .is_ok());

        let r0 = handlebars.render("t0", &people);
        assert_eq!(r0.ok().unwrap(), "BeijingBeijing".to_string());

        let r1 = handlebars.render("t1", &people);
        assert_eq!(r1.ok().unwrap(), "2727".to_string());

        let r2 = handlebars.render("t2", &people);
        assert_eq!(r2.ok().unwrap(), "01".to_string());
    }

    #[test]
    fn test_path_up() {
        let mut handlebars = Registry::new();
        assert!(handlebars
            .register_template_string("t0", "{{#with a}}{{#with b}}{{../../d}}{{/with}}{{/with}}")
            .is_ok());
        let data = btreemap! {
            "a".to_string() => to_json(&btreemap! {
                "b".to_string() => vec![btreemap!{"c".to_string() => vec![1]}]
            }),
            "d".to_string() => to_json(1)
        };

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.ok().unwrap(), "1".to_string());
    }
}
