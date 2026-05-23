# gemini-grpc

Rust gRPC client for Google Gemini and Vertex AI based on Tonic.


### Features

- gRPC transport
- Supports both Gemini API and Vertex AI endpoints
- Streaming and non-streaming generation
- Batteries-included authentication manager
- Typed request/response models
- JSON schema constrained generation


## Authentication
```rust
use gemini::auth::UserAccount;
use gemini::vertex::VertexClient;
let account = UserAccount::from_file("vertex-user.json")?;
let client = VertexClient::new(account, EU_WEST1)?;
```
## Basic Text Generation
```rust
use gemini::vertex::VertexClient;
use gemini::vertex_types::{
    content::Content,
    generate_content_request::GenerateContentRequest,
};

let contents =
    Content::user().with_text("Why is the sky blue?");
let response = client
    .get_client_when_ready()
    .await
    .generate_content(
        GenerateContentRequest::default()
            .with_content(contents)
            .model_string(client.model_string("gemini-2.5-flash-lite")),
    )
    .await?;
println!("{:#?}", response);
```
## Structured Text Generation
```rust
use gemini::vertex_types::{
    content::Content,
    generate_content_request::GenerateContentRequest,
    generation_config::GenerationConfig,
};
let contents =
    Content::user().with_text("You are a book expert and you are recommending books.");
let schema: serde_json::Value =
    serde_json::from_str(include_str!("test_schema.txt"))?;
let schema: gemini::value::Value = schema.into();
let response = client
    .get_client_when_ready()
    .await
    .generate_content(
        GenerateContentRequest::default()
            .with_content(contents)
            .generation_config(
                GenerationConfig::default()
                    .with_json_schema(schema),
            )
            .model_string(client.model_string("gemini-2.5-flash-lite")),
    )
    .await?;
let value: serde_json::Value =
    response.into_inner().deserialize()?;
println!("{value:#?}");
```
## Image generation
```rust
use gemini::vertex_types::{
    content::Content,
    generate_content_request::GenerateContentRequest,
    generation_config::{GenerationConfig, Modality},
    part::Data,
};
let contents = Content::user()
    .with_text("Generate a realistic image of a green apple in front of a white background!");
let image = client
    .get_client()
    .generate_content(
        GenerateContentRequest::default()
            .with_content(contents)
            .model_string(client.model_string("gemini-2.5-flash-image"))
            .generation_config(
                GenerationConfig::default()
                    .with_response_modality(Modality::Image),
            ),
    )
    .await?
    .into_inner();
let response = image.get_single();
if let Some(Data::InlineData(blob)) = response {
    let data: &[u8] = blob.data.as_ref();
    std::fs::write("./output.png", data)?;
}
```
