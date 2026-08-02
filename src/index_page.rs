use crate::consts::{BUILDTIME, GIT_REV};
use askama::Template;
use jiff::tz;
use serde_json;
use std::env;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    git_rev: String,
    tz_json: String,
}

impl IndexTemplate {
    pub fn new() -> Self {
        let tz_json: String = serde_json::to_string(
            &tz::db()
                .available()
                .map(|t| String::from(t.as_str()))
                .collect::<Vec<_>>(),
        )
        .unwrap();

        Self {
            git_rev: env::var("OPENSHIFT_BUILD_COMMIT")
                .unwrap_or(format!("{:.8}", String::from(GIT_REV)))
                .to_string(),
            tz_json,
        }
    }
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self::new()
    }
}
