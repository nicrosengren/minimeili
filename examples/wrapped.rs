use minimeili::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Outer {
    id: u64,
    name: String,
    inner: Inner,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Inner {
    name: String,
}

impl Outer {
    pub fn new(name: impl Into<String>, inner_name: impl Into<String>) -> Self {
        let name = name.into();
        let inner_name: String = inner_name.into();

        let id = name
            .chars()
            .chain(inner_name.chars())
            .map(|c| c as u64)
            .sum();
        Self {
            id,
            name,
            inner: Inner { name: inner_name },
        }
    }
}

impl HasIndex for Outer {
    const INDEX_UID: &'static str = "innerouter";
    const PRIMARY_KEY: &'static str = "id";

    const SEARCHABLE_ATTRIBUTES: &'static [&'static str] = &["name", "inner.name"];

    const SORTABLE_ATTRIBUTES: &'static [&'static str] = &["name"];

    const FILTERABLE_ATTRIBUTES: &'static [&'static str] = &["id", "name"];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let task_ref = Outer::delete_index(&client).await?;
    task_ref.wait_until_stopped(&client).await?;

    Outer::ensure_index(&client).await?;

    Outer::new("Adam", "Bertil").add_to_index(&client).await?;

    task_ref.wait_until_stopped(&client).await?;

    println!("{:#?}", Outer::search(&client, "Adam").await?);
    println!();
    println!("{:#?}", Outer::search(&client, "Bertil").await?);

    Ok(())
}
