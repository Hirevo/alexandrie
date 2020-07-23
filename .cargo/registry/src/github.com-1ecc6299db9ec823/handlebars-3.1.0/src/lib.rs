#![doc(html_root_url = "https://docs.rs/handlebars/3.1.0")]
//! # Handlebars
//!
//! [Handlebars](http://handlebarsjs.com/) is a modern and extensible templating solution originally created in the JavaScript world. It's used by many popular frameworks like [Ember.js](http://emberjs.com) and Chaplin. It's also ported to some other platforms such as [Java](https://github.com/jknack/handlebars.java).
//!
//! And this is handlebars Rust implementation, designed for general purpose text generation.
//!
//! ## Quick Start
//!
//! ```
//! use std::collections::BTreeMap;
//! use handlebars::Handlebars;
//!
//! fn main() {
//!   // create the handlebars registry
//!   let mut handlebars = Handlebars::new();
//!
//!   // register the template. The template string will be verified and compiled.
//!   let source = "hello {{world}}";
//!   assert!(handlebars.register_template_string("t1", source).is_ok());
//!
//!   // Prepare some data.
//!   //
//!   // The data type should implements `serde::Serialize`
//!   let mut data = BTreeMap::new();
//!   data.insert("world".to_string(), "世界!".to_string());
//!   assert_eq!(handlebars.render("t1", &data).unwrap(), "hello 世界!");
//! }
//! ```
//!
//! In this example, we created a template registry and registered a template named `t1`.
//! Then we rendered a `BTreeMap` with an entry of key `world`, the result is just what
//! we expected.
//!
//! I recommend you to walk through handlebars.js' [intro page](http://handlebarsjs.com)
//! if you are not quite familiar with the template language itself.
//!
//! ## Features
//!
//! Handlebars is a real-world templating system that you can use to build
//! your application without pain.
//!
//! ### Isolation of Rust and HTML
//!
//! This library doesn't attempt to use some macro magic to allow you to
//! write your template within your rust code. I admit that it's fun to do
//! that but it doesn't fit real-world use case in my opinion.
//!
//! ### Limited but essential control structure built-in
//!
//! Only essential control directive `if` and `each` were built-in. This
//! prevents you to put too much application logic into your template.
//!
//! ### Extensible helper system
//!
//! You can write your own helper with Rust! It can be a block helper or
//! inline helper. Put you logic into the helper and don't repeat
//! yourself.
//!
//! The built-in helpers like `if` and `each` were written with these
//! helper APIs and the APIs are fully available to developers.
//!
//! ### Template inheritance
//!
//! Every time I look into a templating system, I will investigate its
//! support for [template inheritance][t].
//!
//! [t]: https://docs.djangoproject.com/en/1.9/ref/templates/language/#template-inheritance
//!
//! Template inclusion is not enough. In most case you will need a skeleton
//! of page as parent (header, footer, etc.), and embed you page into this
//! parent.
//!
//! You can find a real example for template inheritance in
//! `examples/partials.rs`, and templates used by this file.
//!
//! ### Strict mode
//!
//! Handlebars, the language designed to work with JavaScript, has no
//! strict restriction on accessing non-existed fields or index. It
//! generates empty string for such case. However, in Rust we want a
//! little bit strict sometime.
//!
//! By enabling `strict_mode` on handlebars:
//!
//! ```
//! # use handlebars::Handlebars;
//! # let mut handlebars = Handlebars::new();
//! handlebars.set_strict_mode(true);
//! ```
//!
//! You will get a `RenderError` when accessing fields that not exists.
//!
//! ## Limitations
//!
//! ### Compatibility with original JavaScript version
//!
//! This implementation is **not fully compatible** with the original javascript version.
//!
//! First of all, mustache block is not supported. I suggest you to use `#if` and `#each` for
//! same functionality.
//!
//! There are some other minor features missing:
//!
//! * Chained else [#12](https://github.com/sunng87/handlebars-rust/issues/12)
//!
//! Feel free to fire an issue on [github](https://github.com/sunng87/handlebars-rust/issues) if
//! you find missing features.
//!
//! ### Types
//!
//! As a static typed language, it's a little verbose to use handlebars.
//! Handlebars templating language is designed against JSON data type. In rust,
//! we will convert user's structs, vectors or maps into SerDe-Json's `Value` type
//! in order to use in template. You have to make sure your data implements the
//! `Serialize` trait from the [Serde](https://serde.rs) project.
//!
//! ## Usage
//!
//! ### Template Creation and Registration
//!
//! Templates are created from String and registered to `Handlebars` with a name.
//!
//! ```
//! # extern crate handlebars;
//!
//! use handlebars::Handlebars;
//!
//! # fn main() {
//!   let mut handlebars = Handlebars::new();
//!   let source = "hello {{world}}";
//!
//!   assert!(handlebars.register_template_string("t1", source).is_ok())
//! # }
//! ```
//!
//! On registration, the template is parsed, compiled and cached in the registry. So further
//! usage will benefit from the one-time work. Also features like include, inheritance
//! that involves template reference requires you to register those template first with
//! a name so the registry can find it.
//!
//! If you template is small or just to experiment, you can use `render_template` API
//! without registration.
//!
//! ```
//! # use std::error::Error;
//! use handlebars::Handlebars;
//! use std::collections::BTreeMap;
//!
//! # fn main() -> Result<(), Box<Error>> {
//!   let mut handlebars = Handlebars::new();
//!   let source = "hello {{world}}";
//!
//!   let mut data = BTreeMap::new();
//!   data.insert("world".to_string(), "世界!".to_string());
//!   assert_eq!(handlebars.render_template(source, &data)?, "hello 世界!".to_owned());
//! # Ok(())
//! # }
//! ```
//!
//! ### Rendering Something
//!
//! Since handlebars is originally based on JavaScript type system. It supports dynamic features like duck-typing, truthy/falsey values. But for a static language like Rust, this is a little difficult. As a solution, we are using the `serde_json::value::Value` internally for data rendering.
//!
//! That means, if you want to render something, you have to ensure the data type implements the `serde::Serialize` trait. Most rust internal types already have that trait. Use `#derive[Serialize]` for your types to generate default implementation.
//!
//! You can use default `render` function to render a template into `String`. From 0.9, there's `renderw` to render text into anything of `std::io::Write`.
//!
//! ```
//! # use std::error::Error;
//! # #[macro_use]
//! # extern crate serde_derive;
//! # extern crate handlebars;
//!
//! use handlebars::Handlebars;
//!
//! #[derive(Serialize)]
//! struct Person {
//!   name: String,
//!   age: i16,
//! }
//!
//! # fn main() -> Result<(), Box<Error>> {
//!   let source = "Hello, {{name}}";
//!
//!   let mut handlebars = Handlebars::new();
//!   assert!(handlebars.register_template_string("hello", source).is_ok());
//!
//!
//!   let data = Person {
//!       name: "Ning Sun".to_string(),
//!       age: 27
//!   };
//!   assert_eq!(handlebars.render("hello", &data)?, "Hello, Ning Sun".to_owned());
//! # Ok(())
//! # }
//! #
//! ```
//!
//! Or if you don't need the template to be cached or referenced by other ones, you can
//! simply render it without registering.
//!
//! ```
//! # use std::error::Error;
//! # #[macro_use]
//! # extern crate serde_derive;
//! # extern crate handlebars;
//! use handlebars::Handlebars;
//! # #[derive(Serialize)]
//! # struct Person {
//! #  name: String,
//! #  age: i16,
//! # }
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//!   let source = "Hello, {{name}}";
//!
//!   let mut handlebars = Handlebars::new();
//!
//!   let data = Person {
//!       name: "Ning Sun".to_string(),
//!       age: 27
//!   };
//!   assert_eq!(handlebars.render_template("Hello, {{name}}", &data)?,
//!       "Hello, Ning Sun".to_owned());
//! # Ok(())
//! # }
//! ```
//!
//! #### Escaping
//!
//! As per the handlebars spec, output using `{{expression}}` is escaped by default (to be precise, the characters `&"<>` are replaced by their respective html / xml entities). However, since the use cases of a rust template engine are probably a bit more diverse than those of a JavaScript one, this implementation allows the user to supply a custom escape function to be used instead. For more information see the `EscapeFn` type and `Handlebars::register_escape_fn()` method.
//!
//! ### Custom Helper
//!
//! Handlebars is nothing without helpers. You can also create your own helpers with rust. Helpers in handlebars-rust are custom struct implements the `HelperDef` trait, concretely, the `call` function. For your convenience, most of stateless helpers can be implemented as bare functions.
//!
//! ```
//! use std::io::Write;
//! # use std::error::Error;
//! use handlebars::{Handlebars, HelperDef, RenderContext, Helper, Context, JsonRender, HelperResult, Output, RenderError};
//!
//! // implement by a structure impls HelperDef
//! #[derive(Clone, Copy)]
//! struct SimpleHelper;
//!
//! impl HelperDef for SimpleHelper {
//!   fn call<'reg: 'rc, 'rc>(&self, h: &Helper, _: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
//!     let param = h.param(0).unwrap();
//!
//!     out.write("1st helper: ")?;
//!     out.write(param.value().render().as_ref())?;
//!     Ok(())
//!   }
//! }
//!
//! // implement via bare function
//! fn another_simple_helper (h: &Helper, _: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
//!     let param = h.param(0).unwrap();
//!
//!     out.write("2nd helper: ")?;
//!     out.write(param.value().render().as_ref())?;
//!     Ok(())
//! }
//!
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//!   let mut handlebars = Handlebars::new();
//!   handlebars.register_helper("simple-helper", Box::new(SimpleHelper));
//!   handlebars.register_helper("another-simple-helper", Box::new(another_simple_helper));
//!   // via closure
//!   handlebars.register_helper("closure-helper",
//!       Box::new(|h: &Helper, r: &Handlebars, _: &Context, rc: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
//!           let param = h.param(0).ok_or(RenderError::new("param not found"))?;
//!
//!           out.write("3rd helper: ")?;
//!           out.write(param.value().render().as_ref())?;
//!           Ok(())
//!       }));
//!
//!   let tpl = "{{simple-helper 1}}\n{{another-simple-helper 2}}\n{{closure-helper 3}}";
//!   assert_eq!(handlebars.render_template(tpl, &())?,
//!       "1st helper: 1\n2nd helper: 2\n3rd helper: 3".to_owned());
//! # Ok(())
//! # }
//!
//! ```
//!
//! Data available to helper can be found in [Helper](struct.Helper.html). And there are more
//! examples in [HelperDef](trait.HelperDef.html) page.
//!
//! You can learn more about helpers by looking into source code of built-in helpers.
//!
//!
//! ### Script Helper
//!
//! Like our Javascript counterparts, handlebars allows user to define simple helpers with
//! a script language, [rhai](https://docs.rs/crate/rhai/). This can be enabled by
//! turning on `script_helper` feature flag.
//!
//! A sample script:
//!
//! ```handlebars
//! {{percent 0.34 label="%"}}
//! ```
//!
//! ```rhai
//! // percent.rhai
//! // get first parameter from `params` array
//! let value = params[0];
//! // get key  value pair `label` from `hash` map
//! let label = hash["label"];
//!
//! // compute the final string presentation
//! (value * 100).to_string() + label
//! ```
//!
//! A runnable [example](https://github.com/sunng87/handlebars-rust/blob/master/examples/script.rs) can be find in the repo.
//!
//! #### Built-in Helpers
//!
//! * `{{{{raw}}}} ... {{{{/raw}}}}` escape handlebars expression within the block
//! * `{{#if ...}} ... {{else}} ... {{/if}}` if-else block
//! * `{{#unless ...}} ... {{else}} .. {{/unless}}` if-not-else block
//! * `{{#each ...}} ... {{/each}}` iterates over an array or object. Handlebar-rust doesn't support mustache iteration syntax so use this instead.
//! * `{{#with ...}} ... {{/with}}` change current context. Similar to {{#each}}, used for replace corresponding mustache syntax.
//! * `{{lookup ... ...}}` get value from array by `@index` or `@key`
//! * `{{> ...}}` include template with name
//! * `{{log ...}}` log value with rust logger, default level: INFO. Currently you cannot change the level.
//! * Boolean helpers that can be used in `if` as subexpression, for example `{{#if (gt 2 1)}} ...`:
//!   * `eq`
//!   * `ne`
//!   * `gt`
//!   * `gte`
//!   * `lt`
//!   * `lte`
//!   * `and`
//!   * `or`
//!   * `not`
//!
//! ### Template inheritance
//!
//! Handlebars.js' partial system is fully supported in this implementation.
//! Check [example](https://github.com/sunng87/handlebars-rust/blob/master/examples/partials.rs#L49) for details.
//!
//!

#![allow(dead_code)]
#![warn(rust_2018_idioms)]
#![recursion_limit = "200"]

#[cfg(not(feature = "no_logging"))]
#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate quick_error;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;

pub use self::block::{BlockContext, BlockParams};
pub use self::context::Context;
pub use self::decorators::DecoratorDef;
pub use self::error::{RenderError, TemplateError, TemplateFileError, TemplateRenderError};
pub use self::helpers::{HelperDef, HelperResult};
pub use self::json::path::Path;
pub use self::json::value::{to_json, JsonRender, PathAndJson, ScopedJson};
pub use self::output::Output;
pub use self::registry::{html_escape, no_escape, EscapeFn, Registry as Handlebars};
pub use self::render::{Decorator, Evaluable, Helper, RenderContext, Renderable};
pub use self::template::Template;

#[doc(hidden)]
pub use self::serde_json::Value as JsonValue;

#[macro_use]
mod macros;
mod block;
mod context;
mod decorators;
mod error;
mod grammar;
mod helpers;
mod json;
mod output;
mod partial;
mod registry;
mod render;
mod support;
pub mod template;
mod util;
