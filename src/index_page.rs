use crate::consts::{BUILDTIME, GIT_REV};
use std::env;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    buildtime: String,
    git_rev: String,
}

impl IndexTemplate {
    pub fn new() -> Self {
        Self {
            buildtime: String::from(BUILDTIME),
            git_rev: env::var("OPENSHIFT_BUILD_COMMIT").unwrap_or(String::from(GIT_REV))[0..6]
                .to_string(),
        }
    }
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self::new()
    }
}

