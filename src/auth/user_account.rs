use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

use crate::auth::{
    error::{JwtError, UacError},
    token::Claims,
};
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
