use crate::{
    client::Client,
    index::Index,
    search::{Search, SearchResponse},
    task::TaskRef,
    Error, IndexSettings, Result,
};

#[allow(async_fn_in_trait)]
pub trait HasIndex
where
    Self: Sized,
{
    const INDEX_UID: &'static str;
    const PRIMARY_KEY: &'static str;

    const SEARCHABLE_ATTRIBUTES: &'static [&'static str] = &["*"];

    const FILTERABLE_ATTRIBUTES: &'static [&'static str] = &[];
    const SORTABLE_ATTRIBUTES: &'static [&'static str] = &[];

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
        c.search(Self::INDEX_UID, search).await
    }

    async fn get_index(c: &Client) -> Result<Index> {
        c.get_index(Self::INDEX_UID).await
    }

    async fn get_index_settings(c: &Client) -> Result<IndexSettings> {
        c.get_index_settings(Self::INDEX_UID).await
    }

    async fn create_index(c: &Client) -> Result<TaskRef> {
        c.create_index(Self::INDEX_UID, Self::PRIMARY_KEY).await
    }

    async fn delete_index(c: &Client) -> Result<TaskRef> {
        c.delete_index(Self::INDEX_UID).await
    }

    async fn delete_documents(
        c: &Client,
        document_uids: &[impl serde::Serialize],
    ) -> Result<TaskRef> {
        c.delete_documents(Self::INDEX_UID, document_uids).await
    }

    async fn delete_all_documents(c: &Client) -> Result<TaskRef> {
        c.delete_all_documents(Self::INDEX_UID).await
    }

    async fn ensure_index_settings(c: &Client) -> Result<()> {
        let mut settings = Self::get_index_settings(c).await?;

        let mut update_needed = false;

        for (local, remote) in [
            (
                Self::FILTERABLE_ATTRIBUTES,
                &mut settings.filterable_attributes,
            ),
            (Self::SORTABLE_ATTRIBUTES, &mut settings.sortable_attributes),
            (
                Self::SEARCHABLE_ATTRIBUTES,
                &mut settings.searchable_attributes,
            ),
        ] {
            let mut l = local.iter().map(|s| String::from(*s)).collect::<Vec<_>>();
            l.sort();
            remote.sort();

            if &l != remote {
                update_needed = true;
                *remote = l;
            }
        }

        if update_needed {
            let task_ref = c.update_index_settings(Self::INDEX_UID, &settings).await?;
            task_ref.wait_until_stopped(c).await?;
            return Ok(());
        }
        Ok(())
    }

    async fn ensure_index(c: &Client) -> Result<()> {
        match Self::get_index(c).await {
            Err(Error::UnexpectedNok { code: 404, .. }) => {
                let task = Self::create_index(c).await?;
                task.wait_until_stopped(c).await?;
            }

            Err(err) => return Err(err),

            Ok(_) => (),
        }

        Self::ensure_index_settings(c).await?;

        Ok(())
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
