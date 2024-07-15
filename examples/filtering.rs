use minimeili::{prelude::*, Search};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct User {
    id: u64,
    name: String,

    sites: Vec<SiteRef>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SiteRef {
    id: u64,
}

impl User {
    pub fn new(name: impl Into<String>, site_ids: impl IntoIterator<Item = u64>) -> Self {
        let name: String = name.into();

        let id = name.chars().map(|c| c as u64).sum();

        Self {
            id,
            name,
            sites: site_ids.into_iter().map(|id| SiteRef { id }).collect(),
        }
    }
}

impl HasIndex for User {
    const INDEX_UID: &'static str = "users";
    const PRIMARY_KEY: &'static str = "id";

    const SEARCHABLE_ATTRIBUTES: &'static [&'static str] = &["name"];

    const SORTABLE_ATTRIBUTES: &'static [&'static str] = &["name"];

    const FILTERABLE_ATTRIBUTES: &'static [&'static str] = &["sites.id"];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = minimeili::Client::from_env();

    let task_ref = User::delete_index(&client).await?;
    task_ref.wait_until_stopped(&client).await?;

    User::ensure_index(&client).await?;

    User::new("Adam", [23, 24, 25])
        .replace_in_index(&client)
        .await?
        .wait_until_stopped(&client)
        .await?;

    User::new("Bertil", [36, 37, 38])
        .replace_in_index(&client)
        .await?
        .wait_until_stopped(&client)
        .await?;

    println!(
        "Filter on 23:\n:{:#?}",
        User::search(&client, Search::new("").filter(Some("sites.id = 23"))).await?
    );
    println!();
    println!(
        "Filter on 36:\n{:#?}",
        User::search(&client, Search::new("").filter(Some("sites.id = 36"))).await?
    );

    Ok(())
}
