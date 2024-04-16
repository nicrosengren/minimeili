#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    client
        .delete_all_documents("names")
        .await?
        .wait_until_stopped(&client)
        .await?;

    Ok(())
}
