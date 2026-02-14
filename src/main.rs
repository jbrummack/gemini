use std::time::Duration;

use gemini::{
    EU_WEST1, UserAccount, VertexClient,
    vertex_types::{Content, GenerateContentRequest, GenerationConfig},
};

#[ctor::ctor]
fn crypto() {
    rustls::crypto::ring::default_provider().install_default();
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let account = UserAccount::from_file("vertex-user.json")?;
    let client = VertexClient::new(account, EU_WEST1)?;

    let contents =
        Content::user().with_text("You are a book expert and you are recommending books.");
    let schema: serde_json::Value = serde_json::from_str(include_str!("test_schema.txt"))?;
    let schema: gemini::value::Value = schema.into();
    let res = client
        .get_client_when_ready()
        .await
        .generate_content(
            GenerateContentRequest::default()
                .with_content(contents)
                .generation_config(GenerationConfig::default().with_json_schema(schema))
                .model_string(client.model_string("gemini-2.5-flash-lite")),
        )
        .await
        .inspect_err(|e| println!("{:?}\n{:?}\n{:?}", e.code(), e.message(), e.metadata()))?;
    println!("{res:#?}");
    let value: serde_json::Value = res.into_inner().deserialize()?;
    println!("{value:#?}");

    Ok(())
}
