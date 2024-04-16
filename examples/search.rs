#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let index_uid = std::env::args()
        .nth(1)
        .expect("first argument should be index uid");

    let qry = std::env::args()
        .nth(2)
        .expect("second argument should be search query");

    let res = client.search::<serde_json::Value>(index_uid, qry).await?;

    println!("{res:#?}");

    Ok(())
}
