use minimeili::prelude::*;

struct TestDocument;

impl HasIndex for TestDocument {
    const INDEX_UID: &'static str = "testingdocs";
    const PRIMARY_KEY: &'static str = "id";

    const SORTABLE_ATTRIBUTES: &'static [&'static str] = &["id", "name"];
    const FILTERABLE_ATTRIBUTES: &'static [&'static str] = &["id", "name"];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    TestDocument::ensure_index(&client).await?;

    Ok(())
}
