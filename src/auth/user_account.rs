use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

use crate::auth::{
    error::{JwtError, UacError},
    token::Claims,
};
#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct ImpersonatedAccount {
    pub(crate) service_account_impersonation_url: String,
    pub(crate) source_credentials: SourceCredentials,
    pub(crate) project_id: Option<String>,
}
#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct SourceCredentials {
    pub(crate) account: String,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) refresh_token: String,
    pub(crate) r#type: String,
    pub(crate) universe_domain: String,
}

#[derive(Debug, serde::Serialize)]

struct RefreshTokenRequest<'a> {
    grant_type: &'a str,

    refresh_token: &'a str,

    client_id: &'a str,

    client_secret: &'a str,
}

#[derive(Debug, serde::Deserialize)]

struct RefreshTokenResponse {
    access_token: String,

    expires_in: i64,

    token_type: String,
}

#[derive(Debug, serde::Serialize)]

struct ImpersonationRequest<'a> {
    delegates: Vec<String>,

    scope: Vec<&'a str>,
    lifetime: String,
}

#[derive(Debug, serde::Deserialize)]

struct ImpersonationResponse {
    access_token: String,
    expire_time: Option<String>,
}
impl ImpersonatedAccount {
    ///Loads a key file that was downloaded from GCP console
    pub fn from_file(
        path: impl AsRef<Path>,
        project_id: impl Into<String>,
    ) -> Result<Self, UacError> {
        let st = std::fs::read_to_string(&path).map_err(|e| {
            UacError::Io(
                path.as_ref()
                    .to_str()
                    .map(|s| String::from(s))
                    .unwrap_or(String::from("Corrupted path")),
                e,
            )
        })?;
        let mut s: Self = serde_json::from_str(&st)?;
        s.project_id = Some(project_id.into());
        Ok(s)
    }
}

///User Account for GCP
#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct UserAccount {
    pub(crate) client_email: String,
    pub(crate) private_key: String,
    pub(crate) project_id: String,
    pub(crate) auth_uri: String,
    pub(crate) token_uri: String,
}
impl UserAccount {
    ///Loads a key file that was downloaded from GCP console
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, UacError> {
        let st = std::fs::read_to_string(&path).map_err(|e| {
            UacError::Io(
                path.as_ref()
                    .to_str()
                    .map(|s| String::from(s))
                    .unwrap_or(String::from("Corrupted path")),
                e,
            )
        })?;
        let s: Self = serde_json::from_str(&st)?;
        Ok(s)
    }
}
impl UserAccount {
    pub fn create_jwt(&self) -> Result<String, JwtError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let claims = Claims {
            iss: self.client_email.as_ref(),
            scope: "https://www.googleapis.com/auth/cloud-platform",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now + 3600,
        };

        let key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key)?;

        Ok(jwt)
    }
}
