use crate::{
    client::Client,
    index::Index,
    search::{Search, SearchResponse},
    task::TaskRef,
    Error, Result,
};

#[allow(async_fn_in_trait)]
pub trait HasIndex
where
    Self: Sized,
{
    const INDEX_UID: &'static str;
    const PRIMARY_KEY: &'static str;

    async fn add_to_index(&self, c: &Client) -> Result<TaskRef>
    where
        Self: serde::Serialize,
    {
        c.index_document(self).await
    }

    async fn search(c: &Client, search: impl Into<Search>) -> Result<SearchResponse<Self>>
    where
        Self: serde::de::DeserializeOwned,
    {
        c.search(search).await
    }

    async fn get_index(c: &Client) -> Result<Index> {
        c.get_index(Self::INDEX_UID).await
    }

    async fn create_index(c: &Client) -> Result<TaskRef> {
        c.create_index(Self::INDEX_UID, Self::PRIMARY_KEY).await
    }

    async fn delete_index(c: &Client) -> Result<TaskRef> {
        c.delete_index(Self::INDEX_UID).await
    }

    async fn ensure_index(c: &Client) -> Result<Option<TaskRef>> {
        if let Err(Error::UnexpectedNok(404)) = Self::get_index(c).await {
            let task = Self::create_index(c).await?;
            return Ok(Some(task));
        }
        Ok(None)
    }
}

pub trait HasIndexExt {
    #[allow(async_fn_in_trait)]
    async fn add_to_index(&self, c: &Client) -> Result<TaskRef>;
}

impl<'a, T> HasIndexExt for &'a [T]
where
    T: HasIndex,
    T: serde::Serialize,
{
    async fn add_to_index(&self, c: &Client) -> Result<TaskRef> {
        c.index_documents(self).await
    }
}
