use std::sync::Arc;

use tonic::{
    Status,
    metadata::MetadataValue,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, ClientTlsConfig},
};

use crate::{
    auth::error::NetConnError,
    google::ai::generativelanguage::v1::generative_service_client::GenerativeServiceClient,
};
#[derive(Clone)]
pub struct ApiKeyInterceptor(Arc<String>);

impl Interceptor for ApiKeyInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let api_key = MetadataValue::try_from(self.0.as_str())
            .map_err(|_| Status::invalid_argument("Invalid API key format"))?;

        request.metadata_mut().insert("x-goog-api-key", api_key);

        Ok(request)
    }
}
pub struct GeminiClient {
    api_key: ApiKeyInterceptor,
    channel: Channel,
}
impl GeminiClient {
    pub fn new(api_key: String, url: &'static str) -> Result<Self, NetConnError> {
        let api_key = ApiKeyInterceptor(Arc::new(api_key));
        let tls = ClientTlsConfig::new().with_enabled_roots();

        let channel = Channel::from_static(url).tls_config(tls)?.connect_lazy();
        Ok(Self { api_key, channel })
    }
    pub fn get_client(
        &self,
    ) -> GenerativeServiceClient<InterceptedService<Channel, ApiKeyInterceptor>> {
        let client =
            GenerativeServiceClient::with_interceptor(self.channel.clone(), self.api_key.clone());
        client
    }
    ///This does the same as get_client and exists for interface compatibility with VertexClient
    pub async fn get_client_when_ready(
        &self,
    ) -> GenerativeServiceClient<InterceptedService<Channel, ApiKeyInterceptor>> {
        self.get_client()
    }
}
