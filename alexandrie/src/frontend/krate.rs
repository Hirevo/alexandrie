use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use alexandrie_index::Indexer;
use alexandrie_storage::Store;

use crate::db::models::{Badge, Crate, CrateAuthor, CrateCategory, CrateKeyword, Keyword};
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BadgeRepr {
    src: String,
    alt: String,
    href: Option<String>,
}

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let name = req.param("crate")?.to_string();

    let user = req.get_author();
    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        //? Get this crate's data.
        let crate_desc = crates::table
            .filter(crates::name.eq(&name))
            .first::<Crate>(conn)
            .optional()?;
        let crate_desc = match crate_desc {
            Some(crate_desc) => crate_desc,
            None => {
                return utils::response::error_html(
                    state.as_ref(),
                    user,
                    StatusCode::NotFound,
                    format!("No crate named '{0}' has been found.", name),
                );
            }
        };
        let krate = state.index.latest_record(&crate_desc.name)?;

        //? Get the HTML-rendered README page of this crate.
        let rendered_readme = state
            .storage
            .get_readme(&crate_desc.name, krate.vers.clone())
            .ok();

        //? Get the authors' names of this crate.
        let authors = CrateAuthor::belonging_to(&crate_desc)
            .inner_join(authors::table)
            .select(authors::name)
            .load::<String>(conn)?;

        //? Get the keywords for this crate.
        let keywords = CrateKeyword::belonging_to(&crate_desc)
            .inner_join(keywords::table)
            .select(keywords::all_columns)
            .load::<Keyword>(conn)?;

        //? Get the categories of this crate.
        let categories = CrateCategory::belonging_to(&crate_desc)
            .inner_join(categories::table)
            .select(categories::name)
            .load::<String>(conn)?;

        //? Get the badges of this crate.
        let badges = Badge::belonging_to(&crate_desc).load::<Badge>(conn)?;
        let badges: Vec<BadgeRepr> = badges
            .into_iter()
            .filter_map(|badge| match badge.badge_type.as_str() {
                "appveyor" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                        service: Option<String>,
                        project_name: Option<String>,
                        id: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));
                    let service = params.service.unwrap_or_else(|| String::from("github"));
                    let project_name = params
                        .project_name
                        .unwrap_or_else(|| repository.replace('.', "-").replace('_', "-"));

                    let src = if let Some(id) = params.id {
                        format!(
                            "https://ci.appveyor.com/api/projects/status/{}/branch/{}?svg=true",
                            id, branch,
                        )
                    } else {
                        format!(
                            "https://ci.appveyor.com/api/projects/status/{}/{}?branch={}&svg=true",
                            service, repository, branch,
                        )
                    };
                    let alt = format!("Appveyor build status for the {} branch", branch);
                    let href = Some(format!("https://ci.appveyor.com/project/{}", project_name));
                    Some(BadgeRepr { src, alt, href })
                }
                "circle-ci" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));

                    let src = format!(
                        "https://circleci.com/gh/{}/tree/{}.svg?style=shield",
                        repository, branch
                    );
                    let alt = format!("Circle CI build status for the {} branch", branch);
                    let href = Some(format!("https://circleci.com/gh/{}", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "cirrus-ci" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));

                    let src = format!(
                        "https://api.cirrus-ci.com/github/{}.svg?branch={}",
                        repository, branch
                    );
                    let alt = format!("Cirrus CI build status for the {} branch", branch);
                    let href = Some(format!("https://cirrus-ci.com/github/{}", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "gitlab" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));

                    let src = format!(
                        "https://gitlab.com/{}/badges/{}/pipeline.svg",
                        repository, branch
                    );
                    let alt = format!("GitLab build status for the {} branch", branch);
                    let href = Some(format!("https://gitlab.com/{}/pipelines", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "azure-devops" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        project: String,
                        pipeline: String,
                        build: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let project = params.project;
                    let pipeline = params.pipeline;
                    let build = params.build.unwrap_or_else(|| String::from("1"));

                    let src = format!(
                        "https://dev.azure.com/{}/_apis/build/status/{}",
                        project, pipeline
                    );
                    let alt = format!("Azure Devops build status for the {} pipeline", pipeline);
                    let href = Some(format!(
                        "https://dev.azure.com/{}/_build/latest?definitionId={}",
                        project, build
                    ));
                    Some(BadgeRepr { src, alt, href })
                }
                "travis-ci" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));

                    let src = format!("https://travis-ci.org/{}.svg?branch={}", repository, branch);
                    let alt = format!("Travis CI build status for the {} branch", branch);
                    let href = Some(format!("https://travis-ci.org/{}", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "codecov" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                        service: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));
                    let service = params.service.unwrap_or_else(|| String::from("github"));

                    let src = format!(
                        "https://codecov.io/{}/{}/coverage.svg?branch={}",
                        service, repository, branch
                    );
                    let alt = format!("CodeCov coverage status for the {} branch", branch);
                    let href = Some(format!(
                        "https://codecov.io/{}/{}?branch={}",
                        service, repository, branch
                    ));
                    Some(BadgeRepr { src, alt, href })
                }
                "coveralls" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                        branch: Option<String>,
                        service: Option<String>,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;
                    let branch = params.branch.unwrap_or_else(|| String::from("master"));
                    let service = params.service.unwrap_or_else(|| String::from("github"));

                    let src = format!(
                        "https://coveralls.io/repos/{}/{}/badge.svg?branch={}",
                        service, repository, branch
                    );
                    let alt = format!("Coveralls coverage status for the {} branch", branch);
                    let href = Some(format!(
                        "https://coveralls.io/{}/{}?branch={}",
                        service, repository, branch
                    ));
                    Some(BadgeRepr { src, alt, href })
                }
                "is-it-maintained-issue-resolution" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;

                    let src = format!(
                        "https://isitmaintained.com/badge/resolution/{}.svg",
                        repository
                    );
                    let alt = format!("Is It Maintained average time to resolve an issue");
                    let href = Some(format!("https://isitmaintained.com/project/{}", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "is-it-maintained-open-issues" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        repository: String,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let repository = params.repository;

                    let src = format!("https://isitmaintained.com/badge/open/{}.svg", repository);
                    let alt = format!("Is It Maintained percentage of issues still open");
                    let href = Some(format!("https://isitmaintained.com/project/{}", repository));
                    Some(BadgeRepr { src, alt, href })
                }
                "maintenance" => {
                    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
                    struct Params {
                        status: String,
                    }
                    let params: Params = json::from_str(badge.params.as_str()).ok()?;

                    let status = params.status.replace('-', "--");
                    let color = match status.as_str() {
                        "actively-developed" => "brightgreen",
                        "passively-maintained" => "yellowgreen",
                        "as-is" => "yellow",
                        "experimental" => "blue",
                        "looking-for-maintainer" => "orange",
                        "deprecated" => "red",
                        _ => "blue",
                    };

                    let src = format!(
                        "https://img.shields.io/badge/maintenance-{}-{}.svg",
                        status, color
                    );
                    let alt = format!("Maintenance intention for this crate");
                    let href = None;
                    Some(BadgeRepr { src, alt, href })
                }
                _ => None,
            })
            .collect();

        let created_at =
            chrono::NaiveDateTime::parse_from_str(crate_desc.created_at.as_str(), DATETIME_FORMAT)
                .unwrap();
        let updated_at =
            chrono::NaiveDateTime::parse_from_str(crate_desc.updated_at.as_str(), DATETIME_FORMAT)
                .unwrap();

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "crate": {
                "id": crate_desc.id,
                "name": crate_desc.name,
                "version": krate.vers,
                "description": crate_desc.description,
                "downloads": helpers::humanize_number(crate_desc.downloads),
                "created_at": helpers::humanize_datetime(created_at),
                "updated_at": helpers::humanize_datetime(updated_at),
                "documentation": crate_desc.documentation,
                "repository": crate_desc.repository,
                "yanked": krate.yanked,
            },
            "badges": badges,
            "authors": authors,
            "rendered_readme": rendered_readme,
            "keywords": keywords,
            "categories": categories,
        });
        Ok(utils::response::html(engine.render("crate", &context)?))
    });

    transaction.await
}
