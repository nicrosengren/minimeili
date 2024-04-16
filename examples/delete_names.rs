#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ids = std::env::args()
        .nth(1)
        .expect("provide document ids as commaseparated first argument")
        .split(',')
        .map(|s| s.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?;

    let client = minimeili::Client::from_env();

    client
        .delete_documents("names", &ids)
        .await?
        .wait_until_stopped(&client)
        .await?;

    Ok(())
}
