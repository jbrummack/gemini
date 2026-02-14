use std::sync::Arc;

use tonic::{
    Status,
    metadata::MetadataValue,
    service::{Interceptor, interceptor::InterceptedService},
    transport::Channel,
};

use crate::google::ai::generativelanguage::v1::generative_service_client::GenerativeServiceClient;
#[derive(Clone)]
pub struct ApiKeyInterceptor(Arc<String>);

impl Interceptor for ApiKeyInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        // 1. Parse the API key into a MetadataValue
        // We use "from_str" here; ensure the key doesn't contain invalid characters
        let api_key = MetadataValue::try_from(self.0.as_str())
            .map_err(|_| Status::invalid_argument("Invalid API key format"))?;

        // 2. Insert the "x-goog-api-key" header
        request.metadata_mut().insert("x-goog-api-key", api_key);

        Ok(request)
    }
}
pub struct GeminiClient {
    api_key: ApiKeyInterceptor,
    channel: Channel,
}
impl GeminiClient {
    pub fn new() -> Self {
        todo!()
    }
    pub fn get_client(
        &self,
    ) -> GenerativeServiceClient<InterceptedService<Channel, ApiKeyInterceptor>> {
        let client =
            GenerativeServiceClient::with_interceptor(self.channel.clone(), self.api_key.clone());
        client
    }
}
