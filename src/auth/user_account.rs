use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

use crate::auth::{
    error::{JwtError, UacError},
    token::Claims,
};
//get_local_private_key()?
impl WorkloadIdentityAccount {
    pub fn create_subject_token() -> Result<String, JwtError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let claims = Claims {
            iss: "my-vps",
            //sub: "vps-1",
            scope: "https://www.googleapis.com/auth/cloud-platform",
            aud: "google-wif",
            iat: now as usize,
            exp: (now + 300) as usize, // IMPORTANT: short-lived (5 min)
        };

        let key = EncodingKey::from_rsa_pem(&[])?;

        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key)?;

        Ok(jwt)
    }
}
#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct CredentialSource {
    /// File path containing the subject token (e.g. OIDC token mounted by k8s/CI)
    pub(crate) file: Option<String>,
    /// URL to fetch the subject token from (e.g. cloud provider metadata server)
    pub(crate) url: Option<String>,
    /// Optional headers for the URL request
    pub(crate) headers: Option<std::collections::HashMap<String, String>>,
}
#[allow(unused)]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct WorkloadIdentityAccount {
    #[serde(default)]
    pub(crate) project_id: String,
    pub(crate) audience: String,
    pub(crate) subject_token_type: String,
    pub(crate) token_url: String,
    pub(crate) credential_source: CredentialSource,
    /// If set, exchanged token is used to impersonate this service account
    /// e.g. "https://iam.googleapis.com/v1/projects/-/serviceAccounts/sa@project.iam.gserviceaccount.com:generateAccessToken"
    pub(crate) service_account_impersonation_url: Option<String>,
}
#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
struct StsTokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
struct ImpersonationTokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expireTime")]
    expire_time: String,
}

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
