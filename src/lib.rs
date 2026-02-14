#[allow(unused)]
mod google {
    pub mod r#type {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/generated/google.r#type.rs"
        ));
    }
    pub mod cloud {
        pub mod aiplatform {
            pub mod v1 {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/generated/google.cloud.aiplatform.v1.rs"
                ));
            }
        }
    }
    pub mod ai {
        pub mod generativelanguage {
            pub mod v1 {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/generated/google.ai.generativelanguage.v1.rs"
                ));
            }
        }
    }
    pub mod protobuf {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/generated/google.protobuf.rs"
        ));
    }
    pub mod api {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/generated/google.api.rs"
        ));
    }
}
pub mod gemini_types {
    pub use crate::google::ai::generativelanguage::v1::*;
}
pub mod value {
    pub use crate::google::protobuf::{ListValue, NullValue, Struct, Value, value::Kind};
}
pub mod vertex_types {
    pub use crate::google::cloud::aiplatform::v1::*;
}
pub use crate::auth::user_account::UserAccount;
pub use crate::client::gemini::GeminiClient;
pub use crate::client::vertex::{ModelString, VertexClient};

pub const GENERATIVE_LANGUAGE_URL: &str = "https://generativelanguage.googleapis.com";
pub const VERTEX_AI_EUW: &str = "https://europe-west1-aiplatform.googleapis.com:443";

macro_rules! define_region {
    ($name:ident, $region_id:expr) => {
        pub const $name: Region = Region(
            $region_id,
            concat!("https://", $region_id, "-aiplatform.googleapis.com:443"),
        );
    };
}
define_region!(EU_WEST1, "europe-west1");
pub struct Region(pub &'static str, pub &'static str);

mod auth;
mod client;
mod request;
pub mod schema;
