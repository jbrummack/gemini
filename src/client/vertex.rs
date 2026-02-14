use tonic::{
    service::interceptor::InterceptedService,
    transport::{Channel, ClientTlsConfig},
};

use crate::{
    Region,
    auth::{auth_interceptor::GcpAuthInterceptor, error::NetConnError, user_account::UserAccount},
    google::cloud::aiplatform::v1::prediction_service_client::PredictionServiceClient,
};
pub struct VertexClient {
    region: Region,
    project_id: String,
    interceptor: GcpAuthInterceptor,
    channel: Channel,
}
#[derive(Debug)]
pub struct ModelString(pub(crate) String);
impl VertexClient {
    /*pub fn with_executor<F>( //needed if we'd wanna run this on mobile
        mut self,
        executor: impl Executor<F> + Send + Sync,
    ) -> Result<Self, NetConnError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let tls = ClientTlsConfig::new().with_enabled_roots();

        let channel = Channel::from_static(self.region.1)
            .executor(executor)
            //.map_err(|e| NetConnError::InvalidUri(e.to_string()))?
            .tls_config(tls)?
            .connect_lazy();
        self.channel = channel;
        Ok(self)
    }*/

    pub fn new(account: UserAccount, Region(loc, url): Region) -> Result<Self, NetConnError> {
        //let region: String = loc.into();
        let project_id = account.project_id.clone();
        let interceptor = GcpAuthInterceptor::new(account)?;
        let tls = ClientTlsConfig::new().with_enabled_roots();

        let channel = Channel::from_static(url)
            //.map_err(|e| NetConnError::InvalidUri(e.to_string()))?
            .tls_config(tls)?
            .connect_lazy();
        Ok(Self {
            region: Region(loc, url),
            project_id,
            interceptor,
            channel,
        })
    }

    pub fn model_string(&self, model: impl AsRef<str>) -> ModelString {
        ModelString(format!(
            "projects/{}/locations/{}/publishers/google/models/{}",
            self.project_id,
            self.region.0,
            model.as_ref()
        ))
    }
    ///Gets the client after the GCP-Token has been set.
    pub async fn get_client_when_ready(
        &self,
    ) -> PredictionServiceClient<InterceptedService<Channel, GcpAuthInterceptor>> {
        let client = PredictionServiceClient::with_interceptor(
            self.channel.clone(),
            self.interceptor.clone(),
        );
        self.interceptor.await_ready().await;
        client
    }
    ///Gets the client without waiting for the GCP-Token.
    pub fn get_client(
        &self,
    ) -> PredictionServiceClient<InterceptedService<Channel, GcpAuthInterceptor>> {
        let client = PredictionServiceClient::with_interceptor(
            self.channel.clone(),
            self.interceptor.clone(),
        );
        client
    }
}
