use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use gcp_auth::TokenProvider;
use tonic::service::Interceptor;

pub struct GcpSecurityInterceptor {
    token: Arc<RwLock<Result<gcp_auth::Token, tonic::Status>>>,
    //scopes: &'static [&'static str],
    //client_instantiator: fn() -> Box<dyn TokenProvider>,
}

impl GcpSecurityInterceptor {
    pub async fn new(
        client: fn() -> Result<Box<dyn TokenProvider>, tonic::Status>,
    ) -> anyhow::Result<Self> {
        let cl = (client)()?;
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = cl.token(scopes).await?.as_ref().clone();
        let token = Arc::new(RwLock::new(Ok(token)));
        tokio::spawn({
            let instantiator = client;
            let token = token.clone();

            async move {
                let mut retries = 0;
                loop {
                    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
                    //interval.tick().await;
                    match Self::refresh_token(instantiator, token.clone(), scopes).await {
                        Ok(token) => {
                            wait_for_expiry(token).await;
                        }
                        Err(e) => {
                            retries += 1;
                            tracing::error!("Failed to refresh GCP Token: {e}");
                            if retries > 5 {
                                tokio::time::sleep(Duration::from_mins(5)).await
                            }
                        }
                    }
                }
            }
        });
        Ok(Self { token })
    }
    pub async fn refresh_token(
        client: fn() -> Result<Box<dyn TokenProvider>, tonic::Status>,
        shared_token: Arc<RwLock<Result<gcp_auth::Token, tonic::Status>>>,
        scopes: &'static [&'static str],
    ) -> anyhow::Result<gcp_auth::Token> {
        let client = (client)()?;
        let token = client.token(scopes).await?.as_ref().clone();

        let mut old_token = shared_token
            .write()
            .map_err(|_| anyhow::anyhow!("RWLOCK POISONED"))?;
        *old_token = Ok(token.clone());
        Ok(token)
    }
}
/// checks the expiry date of the token, resolves instanly if the token expired
async fn wait_for_expiry(tok: gcp_auth::Token) {
    let expiration = tok.expires_at().timestamp();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let time_to_wait = expiration - now;
    if time_to_wait < 0 {
    } else {
        tokio::time::sleep(Duration::from_secs(time_to_wait as u64)).await;
    }
}

impl Interceptor for GcpSecurityInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let token = self.token.read().map_err(|_| {
            tracing::error!("Could not create token: Token Mutex Poisoned");
            tonic::Status::internal("Token Interception Failed, Mutex Poisoned")
        })?;
        let token = token.as_ref().map_err(|e| e.clone())?;
        let bearer_token = format!("Bearer {}", token.as_str()); //format!("Bearer {}", "token");
        request.metadata_mut().insert(
            "authorization",
            bearer_token.parse().map_err(|e| {
                tracing::error!("Could not create token: {e}");
                tonic::Status::internal("Unable to authenticate gcp token")
            })?,
        );
        Ok(request)
    }
}
