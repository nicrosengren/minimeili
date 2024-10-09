use minimeili::Search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let index_uid = std::env::args()
        .nth(1)
        .expect("first argument should be index uid");

    let qry = std::env::args()
        .nth(2)
        .expect("second argument should be search query");

    let filter = std::env::args().nth(3);

    let search = Search::new(&qry).filter(filter.as_ref());

    let res = client
        .search::<serde_json::Value>(index_uid, search)
        .await?;

    println!("{res:#?}");
    println!("query: `{qry}`");
    println!("filter: `{filter:?}`");

    Ok(())
}
