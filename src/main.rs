use gemini::{
    UserAccount, VertexClient,
    region::EU_WEST1,
    vertex_types::{Content, GenerateContentRequest, GenerationConfig},
};

/*#[ctor::ctor]
fn crypto() {
    rustls::crypto::ring::default_provider().install_default();
}*/
//jsonwebtoken::crypto::CryptoProvider::
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
    /*
     use  gemini::vertex_types::part::Data
     let contents = Content::user()
        .with_text("Generate a realistic image of a green apple in front of a white background!");
    let image = client
        .get_client()
        .generate_content(
            GenerateContentRequest::default()
                .with_content(contents)
                .model_string(client.model_string("gemini-2.5-flash-image"))
                .generation_config(GenerationConfig::default().with_response_modality(
                    gemini::vertex_types::generation_config::Modality::Image,
                )),
        )
        .await?
        .into_inner();
    let response = image.get_single();
    if let Some(Data::InlineData(blob)) = response {
        println!("{}", &blob.mime_type);
        let data: &[u8] = blob.data.as_ref();
        std::fs::write("./output_img3.png", data)?;
    }*/
    Ok(())
}
