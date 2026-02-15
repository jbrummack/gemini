use crate::ModelString;

pub trait GenerateContentBuilder {
    type SelfType;
    type Content: ContentBuilder;
    type GenerationConfig: GenerationConfigBuilder;
    fn model_string(self, ms: ModelString) -> Self::SelfType;
    fn model(
        self,
        project_id: impl AsRef<str>,
        location: impl AsRef<str>,
        model_name: impl AsRef<str>,
    ) -> Self::SelfType;
    fn generation_config(self, gen_cfg: Self::GenerationConfig) -> Self::SelfType;
    fn with_content(self, contents: Self::Content) -> Self::SelfType;
}
pub trait ContentBuilder {
    type SelfType;
}
pub trait GenerationConfigBuilder {
    type SelfType;
    fn with_json_schema(self, schema: impl Into<crate::google::protobuf::Value>) -> Self::SelfType;
}
pub trait DataBuilder {
    type SelfType;
}

pub trait PartBuilder {
    type SelfType;
    type Data: DataBuilder;
    type Blob;
    fn new(parts: Vec<Self::SelfType>) -> Self::SelfType;
    ///user role
    fn user() -> Self::SelfType;
    ///model role
    fn model() -> Self::SelfType;
    fn with_part(self, data: Self::Data) -> Self::SelfType;
    fn with_data(self, blob: Self::Blob) -> Self::SelfType;
    fn with_image(self, data: Vec<u8>, mime_type: impl Into<String>) -> Self::SelfType;
}
