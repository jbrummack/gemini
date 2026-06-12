use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rustls::ClientConfig;

use crate::auth::error::UacError;
use crate::auth::user_account::{ImpersonatedAccount, WorkloadIdentityAccount};
use crate::auth::{
    error::{JwtError, NetConnError},
    token::Claims,
    user_account::UserAccount,
};
/*pub struct Auth0Req {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}*/
impl WorkloadIdentityAccount {
    pub fn create_jwt(
        client_email: &str,
        private_key_pem: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            iss: client_email,
            scope: "https://www.googleapis.com/auth/cloud-platform",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now + 3600,
        };

        let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        encode(&Header::new(Algorithm::RS256), &claims, &key)
    }
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
        project_id: Option<impl Into<String>>,
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
        if let Some(p) = project_id {
            s.project_id = p.into();
        }
        Ok(s)
    }
    /// Reads the subject token from file or URL as configured in credential_source
    async fn get_subject_token(
        &self,
        client: &Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>,
    ) -> Result<String, NetConnError> {
        if let Some(path) = &self.credential_source.file {
            let token = std::fs::read_to_string(path);
            return match token {
                Ok(token) => Ok(token.trim().to_string()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    Err(NetConnError::MissingCredentialSource)
                }
                Err(e) => Err(NetConnError::Io(e)),
            };
        }

        if let Some(url) = &self.credential_source.url {
            let mut builder = Request::builder().method(Method::GET).uri(url);

            if let Some(headers) = &self.credential_source.headers {
                for (k, v) in headers {
                    builder = builder.header(k.as_str(), v.as_str());
                }
            }

            let req = builder.body(Full::new(Bytes::new()))?;
            let mut res = client.request(req).await?;
            let body_bytes = res.body_mut().collect().await?.to_bytes();
            let token = String::from_utf8_lossy(&body_bytes).trim().to_string();
            return Ok(token);
        }

        Err(NetConnError::MissingCredentialSource)
    }
}
impl UserAccount {
    pub fn create_jwt_hyper(&self) -> Result<String, JwtError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let claims = Claims {
            iss: self.client_email.as_ref(),
            scope: "https://www.googleapis.com/auth/cloud-platform",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now + 3600,
        };
        let key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;
        Ok(encode(&Header::new(Algorithm::RS256), &claims, &key)?)
    }
}
pub enum FetchAccount {
    UserAccount(UserAccount),
    ImpersonatedAccount(ImpersonatedAccount),
    WorkloadIdentity(WorkloadIdentityAccount),
}
pub struct TokenFetcher {
    account: FetchAccount,
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>,
}

#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
struct StsTokenResponse {
    error: Option<String>,
    error_description: Option<String>,
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
#[derive(serde::Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}
impl TokenFetcher {
    pub async fn fetch_provider_token(
        &self,
        in_path: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        url: impl AsRef<str>,
    ) -> Result<(), NetConnError> {
        let body = std::fs::read(in_path)?;
        let req = Request::builder()
            .uri(url.as_ref())
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from_owner(body)))?;
        let res = self.client.request(req).await?;
        let (p, data) = res.into_parts();
        if !p.status.is_success() {
            return Err(NetConnError::Status(p.status));
        }
        let collect = data.collect().await?.to_bytes();
        std::fs::write(out_path, collect)?;
        Ok(())
    }
    /// Existing `fetch` match should add this arm:
    ///
    /// FetchAccount::WorkloadIdentity(wi_account) => self.fetch_workload_identity(wi_account).await,
    async fn fetch_workload_identity(
        &self,
        account: &WorkloadIdentityAccount,
    ) -> Result<TokenResponse, NetConnError> {
        // Step 1: get the external subject token
        let subject_token = account.get_subject_token(&self.client).await?;

        // Step 2: exchange it via STS for a federated GCP access token
        let body_str = format!(
            "grant_type=urn:ietf:params:oauth:grant-type:token-exchange\
             &audience={}\
             &scope=https://www.googleapis.com/auth/cloud-platform\
             &requested_token_type=urn:ietf:params:oauth:token-type:access_token\
             &subject_token_type={}\
             &subject_token={}",
            urlencoding::encode(&account.audience),
            urlencoding::encode(&account.subject_token_type),
            urlencoding::encode(&subject_token),
        );

        let req = Request::builder()
            .method(Method::POST)
            .uri(&account.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Full::new(Bytes::from(body_str)))?;
        let mut res = self.client.request(req).await?;
        let body_bytes = res.body_mut().collect().await?.to_bytes();
        let sts_resp = String::from_utf8_lossy(body_bytes.as_ref());
        println!("{sts_resp}");
        let sts_token: StsTokenResponse = serde_json::from_slice(&body_bytes)?;

        // Step 3: optionally impersonate a service account using the federated token
        match &account.service_account_impersonation_url {
            None => Ok(TokenResponse {
                access_token: sts_token.access_token,
                expires_in: sts_token.expires_in,
                token_type: sts_token.token_type,
            }),
            Some(impersonation_url) => {
                let body = serde_json::json!({
                    "scope": ["https://www.googleapis.com/auth/cloud-platform"],
                    "lifetime": "3600s",
                });
                let body_bytes_req = serde_json::to_vec(&body)?;

                let req = Request::builder()
                    .method(Method::POST)
                    .uri(impersonation_url)
                    .header("Content-Type", "application/json; charset=utf-8")
                    .header(
                        "Authorization",
                        format!("Bearer {}", sts_token.access_token),
                    )
                    .body(Full::new(Bytes::from(body_bytes_req)))?;

                let mut res = self.client.request(req).await?;
                let body_bytes = res.body_mut().collect().await?.to_bytes();
                let imp_token: ImpersonationTokenResponse = serde_json::from_slice(&body_bytes)?;

                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                let expire_time = chrono::DateTime::parse_from_rfc3339(&imp_token.expire_time)
                    .map(|dt| dt.timestamp() as u64)
                    .unwrap_or(now + 3600);
                let expires_in = expire_time.saturating_sub(now);

                Ok(TokenResponse {
                    access_token: imp_token.access_token,
                    expires_in,
                    token_type: "Bearer".to_string(),
                })
            }
        }
    }
}

impl TokenFetcher {
    pub fn new(account: FetchAccount) -> Result<Self, NetConnError> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let https = HttpsConnectorBuilder::new()
            .with_tls_config(config)
            //.with_native_roots()? // Or .with_webpki_roots()
            .https_only()
            .enable_http1() //we run this once in an hour, so http2 doesnt matter
            //.enable_http2()
            .build();

        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);

        Ok(Self {
            account: account,
            client,
        })
    }

    pub async fn fetch(&self) -> Result<TokenResponse, NetConnError> {
        match &self.account {
            FetchAccount::UserAccount(user_account) => {
                let jwt = user_account.create_jwt()?;
                let body_str = format!(
                    "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={}",
                    jwt
                );

                let req = Request::builder()
                    .method(Method::POST)
                    .uri("https://oauth2.googleapis.com/token")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(Full::new(Bytes::from(body_str)))?;

                let mut res = self.client.request(req).await?;

                let body_bytes = res.body_mut().collect().await?.to_bytes();

                let token: TokenResponse = serde_json::from_slice(&body_bytes)?;
                Ok(token)
            }
            FetchAccount::ImpersonatedAccount(impersonated_account) => {
                let creds = &impersonated_account.source_credentials;
                let body_str = format!(
                    "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
                    urlencoding::encode(&creds.refresh_token),
                    urlencoding::encode(&creds.client_id),
                    urlencoding::encode(&creds.client_secret),
                );
                let req = Request::builder()
                    .method(Method::POST)
                    .uri("https://oauth2.googleapis.com/token")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(Full::new(Bytes::from(body_str)))?;
                let mut res = self.client.request(req).await?;
                let body_bytes = res.body_mut().collect().await?.to_bytes();

                let token: TokenResponse = serde_json::from_slice(&body_bytes)?;
                Ok(token)
            }
            FetchAccount::WorkloadIdentity(wi_account) => {
                self.fetch_workload_identity(wi_account).await
            }
        }
    }
}
