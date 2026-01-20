use minimeili::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let name = std::env::args()
        .nth(1)
        .expect("provide a name via first argument");

    let id = name.bytes().map(|b| b as u64).sum::<u64>();

    let task_ref = TestDocument { id, name }.replace_in_index(&client).await?;
    println!("adding document {task_ref:?}",);

    println!("wating for task");
    let task = task_ref.wait_until_stopped(&client).await?;

    println!("task: {task:#?}");

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestDocument {
    id: u64,
    name: String,
}

impl minimeili::HasIndex for TestDocument {
    const INDEX_UID: &'static str = "names";
    const PRIMARY_KEY: &'static str = "id";
}
