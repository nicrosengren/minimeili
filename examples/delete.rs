use minimeili::prelude::*;

struct TestDocument;

impl HasIndex for TestDocument {
    const INDEX_UID: &'static str = "testingdocs";
    const PRIMARY_KEY: &'static str = "id";
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    println!(
        "Deleting index: {:?}",
        TestDocument::delete_index(&client).await
    );

    Ok(())
}
