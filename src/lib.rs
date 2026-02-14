pub mod google {
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

pub mod auth;
pub mod client;
pub mod request;
pub mod schema;
