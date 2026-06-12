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
use crate::auth::error::{NetConnError, UacError};
use crate::auth::hyper_fetcher::FetchAccount;
pub use crate::auth::user_account::{ImpersonatedAccount, UserAccount};
pub use crate::client::gemini::GeminiClient;
pub use crate::client::vertex::{ModelString, VertexClient};
impl From<ImpersonatedAccount> for FetchAccount {
    fn from(value: ImpersonatedAccount) -> Self {
        FetchAccount::ImpersonatedAccount(value)
    }
}
impl From<UserAccount> for FetchAccount {
    fn from(value: UserAccount) -> Self {
        FetchAccount::UserAccount(value)
    }
}
pub const GENERATIVE_LANGUAGE_URL: &str = "https://generativelanguage.googleapis.com";
pub const VERTEX_AI_EUW: &str = "https://europe-west1-aiplatform.googleapis.com:443";

pub mod region {
    macro_rules! define_region {
        ($name:ident, $region_id:expr) => {
            pub const $name: Region = Region(
                $region_id,
                concat!("https://", $region_id, "-aiplatform.googleapis.com:443"),
            );
        };
    }
    pub struct Region(pub &'static str, pub &'static str);
    define_region!(EU_WEST1, "europe-west1");
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("User account error: {0}")]
    UserAccount(#[from] UacError),
    #[error("Networking error: {0}")]
    NetConn(#[from] NetConnError),
}
mod auth;
mod client;
mod request;
pub mod schema;
