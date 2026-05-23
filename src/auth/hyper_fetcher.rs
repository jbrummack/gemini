use std::time::{SystemTime, UNIX_EPOCH};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rustls::ClientConfig;

use crate::auth::user_account::ImpersonatedAccount;
use crate::auth::{
    error::{JwtError, NetConnError},
    token::Claims,
    user_account::UserAccount,
};

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
}
pub struct TokenFetcher {
    account: FetchAccount,
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>,
}
#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
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
                //let client = reqwest::Client::new();
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
                // 1. Exchange refresh_token for OAuth access token

                /*let refresh_resp: RefreshTokenResponse = client
                .post("https://oauth2.googleapis.com/token")
                .form(&RefreshTokenRequest {
                    grant_type: "refresh_token",

                    refresh_token: &account.source_credentials.refresh_token,

                    client_id: &account.source_credentials.client_id,

                    client_secret: &account.source_credentials.client_secret,
                })
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;*/

                /*let access_token = refresh_resp.access_token;

                // 2. Call service account impersonation endpoint

                /*let url = format!(
                    "{}:generateAccessToken",
                    account
                        .service_account_impersonation_url
                        .trim_end_matches(':')
                );*/
                let body = serde_json::to_vec(&ImpersonationRequest {
                    delegates: vec![],

                    scope: scopes,

                    lifetime: "3600s".to_string(),
                });
                let req = Request::builder().method(Method::POST)
                let impersonation_resp: ImpersonationResponse = client
                    .post(url)
                    .bearer_auth(access_token)
                    .json()
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;

                Ok(impersonation_resp.access_token)*/
            }
        }
    }
}
