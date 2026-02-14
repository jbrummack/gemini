pub mod gemini;
pub mod vertex;
use crate::google;
use crate::google::ai::generativelanguage::v1 as gemini_path;
use crate::google::cloud::aiplatform::v1 as vertex_path;
#[macro_export(local_inner_macros)]
macro_rules! identical_impl {
    () => {
        impl Into<Part> for Blob {
            fn into(self) -> Part {
                Part::new(Data::InlineData(self))
            }
        }
        impl Into<Part> for &str {
            fn into(self) -> Part {
                Part::new(Data::Text(self.to_string()))
            }
        }
        impl Into<Part> for String {
            fn into(self) -> Part {
                Part::new(Data::Text(self))
            }
        }

        impl Content {
            pub fn new(parts: Vec<Part>) -> Self {
                Self {
                    role: "user".into(),
                    parts,
                }
            }
            ///user role
            pub fn user() -> Self {
                let mut c = Self::default();
                c.role = "user".into();
                c
            }
            ///model role
            pub fn model() -> Self {
                let mut c = Self::default();
                c.role = "model".into();
                c
            }
            pub fn with_jpeg(self, data: Vec<u8>) -> Self {
                self.with_image(data, "image/jpeg")
            }
            pub fn with_png(self, data: Vec<u8>) -> Self {
                self.with_image(data, "image/png")
            }
            pub fn with_webp(self, data: Vec<u8>) -> Self {
                self.with_image(data, "image/webp")
            }
            pub fn with_heic(self, data: Vec<u8>) -> Self {
                self.with_image(data, "image/heic")
            }
            pub fn with_heif(self, data: Vec<u8>) -> Self {
                self.with_image(data, "image/heif")
            }
            pub fn with_image(self, data: Vec<u8>, mime_type: impl Into<String>) -> Self {
                self.with_data(Blob {
                    mime_type: mime_type.into(),
                    data,
                })
            }
            pub fn with_data(self, blob: Blob) -> Self {
                self.with_part(Data::InlineData(blob))
            }
            pub fn with_text(self, text: impl Into<String>) -> Self {
                let text: String = text.into();
                self.with_part(Data::Text(text))
            }
            pub fn with_part(mut self, data: Data) -> Self {
                self.parts.push(Part::new(data));
                self
            }
        }
        impl GenerateContentRequest {
            pub fn model_string(mut self, ModelString(ms): ModelString) -> Self {
                self.model = ms;
                self
            }
            pub fn model(
                mut self,
                project_id: impl AsRef<str>,
                location: impl AsRef<str>,
                model_name: impl AsRef<str>,
            ) -> Self {
                self.model = ::std::format!(
                    "projects/{}/locations/{}/publishers/google/models/{}",
                    project_id.as_ref(),
                    location.as_ref(),
                    model_name.as_ref()
                );
                self
            }
            pub fn generation_config(mut self, gen_cfg: GenerationConfig) -> Self {
                self.generation_config = Some(gen_cfg);
                self
            }
            pub fn with_content(mut self, contents: Content) -> Self {
                self.contents.push(contents);
                self
            }
        }

        impl GenerateContentResponse {
            pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
                for c in &self.candidates {
                    if let Some(content) = &c.content {
                        for p in &content.parts {
                            if let Part {
                                data: Some(Data::Text(text)),
                                ..
                            } = p
                            {
                                let restype: T = serde_json::from_str(&text)?;
                                return Ok(restype);
                            }
                        }
                    }
                }
                Err(serde_json::Error::custom("No valid message in Response!"))
            }
        }
    };
}
impl vertex_path::Part {
    pub fn new(data: vertex_path::part::Data) -> Self {
        Self {
            data: Some(data),
            metadata: None,
            thought: false,
            thought_signature: Vec::with_capacity(0),
            media_resolution: None,
        }
    }
}
impl vertex_path::GenerateContentRequest {
    pub fn with_system_instruction(mut self, sys_inst: vertex_path::Content) -> Self {
        self.system_instruction = Some(sys_inst);
        self
    }
}
impl gemini_path::Part {
    pub fn new(data: gemini_path::part::Data) -> Self {
        Self {
            data: Some(data),
            metadata: None,
        }
    }
}
impl gemini_path::GenerationConfig {
    pub fn with_json_schema(mut self, schema: impl Into<google::protobuf::Value>) -> Self {
        let schema: google::protobuf::Value = schema.into();
        self.response_json_schema_ordered = Some(schema);
        self
    }
}
impl vertex_path::GenerationConfig {
    pub fn with_json_schema(mut self, schema: impl Into<google::protobuf::Value>) -> Self {
        let schema: google::protobuf::Value = schema.into();
        //Invalid argument: response_mime_type must be set when response_json_schema is set.
        self.response_json_schema = Some(schema);
        self.response_mime_type = "application/json".into();
        self
    }
    pub fn with_schema(mut self, schema: vertex_path::Schema) -> Self {
        self.response_schema = Some(schema);
        self.response_mime_type = "application/json".into();
        self
    }
}
