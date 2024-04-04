use minimeili::prelude::*;

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestDocument {
    id: u64,
    name: String,
}

impl minimeili::HasIndex for TestDocument {
    const INDEX_UID: &'static str = "names";
    const PRIMARY_KEY: &'static str = "id";
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let qry = std::env::args()
        .nth(1)
        .expect("provide search query through first argument");

    println!("{:#?}", TestDocument::search(&client, qry).await);

    Ok(())
}
