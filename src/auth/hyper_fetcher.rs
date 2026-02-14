use std::time::{SystemTime, UNIX_EPOCH};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

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

pub struct TokenFetcher {
    account: UserAccount,
    client: Client<hyper_rustls::HttpsConnector<HttpConnector>, Full<Bytes>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

impl TokenFetcher {
    pub fn new(account: UserAccount) -> Result<Self, NetConnError> {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()? // Or .with_webpki_roots()
            .https_only()
            .enable_http1() //we run this once in an hour, so http2 doesnt matter
            //.enable_http2()
            .build();

        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);

        Ok(Self { account, client })
    }

    pub async fn fetch(&self) -> Result<TokenResponse, NetConnError> {
        let jwt = self.account.create_jwt()?;
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
}
