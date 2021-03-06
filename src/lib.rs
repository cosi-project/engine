pub mod consts;
pub mod machinery;
pub mod unix;

use serde::{Deserialize, Serialize};

pub mod spec {
    pub mod engine {
        include!(concat!(env!("OUT_DIR"), "/proto/engine.rs"));
    }

    pub mod resource {
        include!(concat!(env!("OUT_DIR"), "/proto/resource.rs"));
    }

    pub mod runtime {
        include!(concat!(env!("OUT_DIR"), "/proto/runtime.rs"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInstance {
    pub api: String,
    pub version: String,
    pub r#type: String,
    pub namespace: String,
    pub id: String,
    pub spec: Box<serde_yaml::Value>,
}
