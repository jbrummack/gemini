use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::auth::{error::NetConnError, hyper_fetcher::TokenFetcher, user_account::UserAccount};
///Interceptor that asynchronously fetches a token, which can then be read synchronously.
#[derive(Debug, Clone)]
pub struct GcpAuthInterceptor(Arc<RwLock<Result<String, tonic::Status>>>, Arc<Notify>);

impl GcpAuthInterceptor {
    pub fn set_token(&self, token: Result<String, tonic::Status>) {
        let mut old_token = self.0.write().unwrap();
        *old_token = token;
    }
    ///Await notification from the first token
    pub async fn await_ready(&self) {
        if !self.is_ready() {
            self.1.notified().await;
        }
        tracing::info!("GCP-Token Ready!")
        //println!("Token ready");
    }
    ///Check if the Token is Ok()
    pub fn is_ready(&self) -> bool {
        match self.0.read() {
            Ok(rd) => rd.is_ok(),
            _ => false,
        }
    }
    pub fn new(account: UserAccount) -> Result<Self, NetConnError> {
        let s = Self(
            Arc::new(RwLock::new(Err(tonic::Status::unavailable(
                "Hasnt fetched token yet. If you want to wait for the token, use get_client_when_ready()!",
            )))),
            Arc::new(Notify::new()),
        );
        let fetcher = TokenFetcher::new(account)?;
        tokio::task::spawn({
            let s = s.clone();
            let fetcher = fetcher;
            async move {
                let mut interval = tokio::time::interval(Duration::from_mins(55));
                loop {
                    interval.tick().await;
                    let token = fetcher.fetch().await.map_err(|e| {
                        tonic::Status::internal(format!("Unable to fetch token: {e}"))
                    });
                    s.set_token(token.map(|t| t.access_token));
                    s.1.notify_waiters();
                }
            }
        });
        Ok(s)
    }
}
use tokio::sync::Notify;
use tonic::service::Interceptor;
///Inserts the fetched token
impl Interceptor for GcpAuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let token = self.0.read().map_err(|_| {
            tracing::error!("Could not create token: Token Mutex Poisoned");
            tonic::Status::internal("Token Interception Failed, Mutex Poisoned")
        })?;
        let token = token.as_ref().map_err(|e| e.clone())?;
        let bearer_token = format!("Bearer {}", token.as_str());
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
