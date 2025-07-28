use crate::consts::{BUILDTIME, GIT_REV};
use askama::Template;
use chrono_tz::TZ_VARIANTS;
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
        let tz_json: String =
            serde_json::to_string(&TZ_VARIANTS.iter().map(|t| t.name()).collect::<Vec<_>>())
                .unwrap();

        Self {
            git_rev: env::var("OPENSHIFT_BUILD_COMMIT").unwrap_or(String::from(GIT_REV))[0..6]
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
