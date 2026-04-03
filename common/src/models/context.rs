use std::{collections::HashMap, path::PathBuf};

use crate::parser::GlobalData;

pub struct BuildContext {
    pub base_path: PathBuf,
    pub env: HashMap<String, String>,
}

impl BuildContext {
    pub fn new(base_path: PathBuf, env: HashMap<String, String>) -> Self {
        Self { base_path, env }
    }
}

impl From<GlobalData> for BuildContext {
    fn from(value: GlobalData) -> Self {
        Self {
            base_path: value.project_root,
            env: value.env,
        }
    }
}
