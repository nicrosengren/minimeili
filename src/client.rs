use reqwest::{header, Method, Url};
use std::{env, sync::Arc};

use crate::{
    index::Index,
    search::{Search, SearchResponse},
    task::{AsTaskUid, Task, TaskRef},
    Error, HasIndex, Result,
};

#[derive(Clone)]
pub struct Client {
    c: reqwest::Client,
    base_url: Arc<Url>,
}

trait Payload {
    fn set_to(self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder;
}

trait FromResponse {
    type Output;
    async fn from_response(res: reqwest::Response) -> Result<Self::Output>;
}

struct Json<'a, T>(&'a T);

impl<'a, T> Payload for Json<'a, T>
where
    T: serde::Serialize,
{
    fn set_to(self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.json(self.0)
    }
}

impl<R> FromResponse for Json<'_, R>
where
    R: serde::de::DeserializeOwned,
{
    type Output = R;
    async fn from_response(res: reqwest::Response) -> Result<R> {
        let bs = res.bytes().await?;

        match serde_json::from_slice::<R>(&bs) {
            Ok(res) => Ok(res),
            Err(err) => {
                let body = String::from_utf8_lossy(&bs).to_string();
                Err(Error::Deserialize { err, body })
            }
        }
    }
}

struct Empty;

impl Payload for Empty {
    fn set_to(self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb
    }
}

impl FromResponse for Empty {
    type Output = ();
    async fn from_response(_: reqwest::Response) -> Result<()> {
        Ok(())
    }
}

impl Client {
    fn build_request(&self, m: Method, path: &str) -> reqwest::RequestBuilder {
        let mut url = Url::clone(&self.base_url);
        url.set_path(path);
        self.c.request(m, url)
    }

    async fn req<R>(&self, method: Method, path: &str, payload: impl Payload) -> Result<R::Output>
    where
        R: FromResponse,
    {
        let http_res = payload
            .set_to(self.build_request(method, path))
            .send()
            .await?;

        if http_res.status().is_success() {
            R::from_response(http_res).await
        } else {
            Err(Error::UnexpectedNok(http_res.status().as_u16()))
        }
    }

    pub async fn get_task(&self, task_uid: impl AsTaskUid) -> Result<Task> {
        self.req::<Json<Task>>(
            Method::GET,
            &format!("/tasks/{}", task_uid.as_task_uid()),
            Empty,
        )
        .await
    }

    /// Searches index T
    pub async fn search<T>(&self, search: impl Into<Search>) -> Result<SearchResponse<T>>
    where
        T: HasIndex,
        T: serde::de::DeserializeOwned,
    {
        let search = search.into();
        self.req::<Json<SearchResponse<T>>>(
            Method::POST,
            &format!("/indexes/{}/search", T::INDEX_UID),
            Json(&search),
        )
        .await
    }

    pub async fn index_documents<T>(&self, docs: &[T]) -> Result<TaskRef>
    where
        T: HasIndex,
        T: serde::Serialize,
    {
        self.req::<Json<TaskRef>>(
            Method::POST,
            &format!("/indexes/{}/documents", T::INDEX_UID),
            Json(&docs),
        )
        .await
    }

    pub async fn index_document<T>(&self, doc: &T) -> Result<TaskRef>
    where
        T: HasIndex,
        T: serde::Serialize,
    {
        self.req::<Json<TaskRef>>(
            Method::POST,
            &format!("/indexes/{}/documents", T::INDEX_UID),
            Json(doc),
        )
        .await
    }

    pub async fn get_index(&self, index_uid: impl AsRef<str>) -> Result<Index> {
        self.req::<Json<Index>>(
            Method::GET,
            &format!("/indexes/{}/documents", index_uid.as_ref()),
            Empty,
        )
        .await
    }

    pub async fn create_index(
        &self,
        index_uid: impl AsRef<str>,
        primary_key: impl AsRef<str>,
    ) -> Result<TaskRef> {
        self.req::<Json<TaskRef>>(
            Method::POST,
            "/indexes",
            Json(&serde_json::json!({
                "uid": index_uid.as_ref(),
                "primaryKey": primary_key.as_ref(),
            })),
        )
        .await
    }

    pub async fn delete_index(&self, index_uid: impl AsRef<str>) -> Result<TaskRef> {
        self.req::<Json<TaskRef>>(
            Method::DELETE,
            &format!("/indexes/{}", index_uid.as_ref()),
            Empty,
        )
        .await
    }

    pub fn new(token: &str, url_s: &str) -> Self {
        let url: reqwest::Url = url_s.parse().expect("invalid URL");

        if url.cannot_be_a_base() {
            panic!("{url} is not a valid base");
        }

        let authorization_header = format!("Bearer {token}");

        let c = reqwest::Client::builder()
            .default_headers(header::HeaderMap::from_iter([(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&authorization_header)
                    .expect("token contained invalid values"),
            )]))
            .use_rustls_tls()
            .build()
            .expect("building client");

        Self {
            c,
            base_url: Arc::new(url),
        }
    }

    #[cfg(feature = "tokio")]
    pub async fn wait_for_task(
        &self,
        task_uid: impl AsTaskUid,
        interval: std::time::Duration,
    ) -> Result<Task> {
        let uid = task_uid.as_task_uid();

        loop {
            tokio::time::sleep(interval).await;
            let task = self.get_task(uid).await?;
            if task.status.has_stopped() {
                return Ok(task);
            }
        }
    }

    /// Creates a client from environment variables:
    ///
    /// * MEILI_TOKEN
    /// * MEILI_URL
    // @TODO move this into a separate function taking params and returning a Result
    pub fn from_env() -> Self {
        let token = env::var("MEILI_TOKEN").expect("environment varaible MEILI_TOKEN");
        let url = env::var("MEILI_URL").expect("environment varaible MEILI_TOKEN");

        Self::new(&token, &url)
    }
}
