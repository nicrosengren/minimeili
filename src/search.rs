use std::collections::HashMap;

use crate::HasIndex;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Search {
    #[serde(rename = "q")]
    query: String,

    /// Number of documents to skip
    /// default 1
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,

    /// Maximum number of documents returned
    /// default 20
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,

    /// Maximum number of documents returned for a page
    /// default 1
    #[serde(skip_serializing_if = "Option::is_none")]
    hits_per_page: Option<u32>,

    /// Request a specific page of results
    /// default 1
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,

    /// Filter queries by an attribute's value
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,

    /// Display the count of matches per facet
    #[serde(skip_serializing_if = "Vec::is_empty")]
    facets: Vec<String>,

    /// Attributes to display in the returned documents
    /// default: ["*"]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attributes_to_retrieve: Vec<String>,

    // Cropping @TODO: extract cropping into own struct
    // which is flatten into Search
    /// Attributes whose values have to be cropped
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attributes_to_crop: Vec<String>,

    /// Attributes whose values have to be cropped
    /// default 10
    #[serde(skip_serializing_if = "Option::is_none")]
    crop_length: Option<u32>,

    /// String marking crop boundaries
    /// default "…"
    #[serde(skip_serializing_if = "Option::is_none")]
    crop_marker: Option<String>,

    // Highlighting, @TODO: extract highlighting into own
    // struct which is flattened into Search
    #[serde(skip_serializing_if = "Vec::is_empty")]
    /// Highlight matching terms contained in an attribute
    attributes_to_highlight: Vec<String>,

    /// String inserted at the start of a highlighted term
    /// default "<em>"
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight_pre_tag: Option<String>,

    /// String inserted at the end of a highlighted term
    /// default "</em>"
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight_post_tag: Option<String>,

    /// Return matching terms location
    /// default false
    #[serde(skip_serializing_if = "Option::is_none")]
    show_matches_position: Option<bool>,

    // @TODO: create trait that allows calling desc asc on str-types
    /// Sort search results by an attribute's value
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sort: Vec<String>,

    /// Strategy used to match query terms within documents
    /// default "last"
    #[serde(skip_serializing_if = "Option::is_none")]
    matching_strategy: Option<String>,

    /// Display the global ranking score of a document
    /// defeault false
    #[serde(skip_serializing_if = "Option::is_none")]
    show_ranking_score: Option<bool>,

    /// Restrict search to the specified attributes
    /// default [ "*" ]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attributes_to_search_on: Vec<String>,
}

impl<T> From<T> for Search
where
    String: From<T>,
{
    fn from(s: T) -> Search {
        Self::new(s)
    }
}

impl Search {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            offset: None,
            limit: None,
            hits_per_page: None,
            page: None,
            filter: None,
            facets: vec![],

            attributes_to_retrieve: vec![],
            attributes_to_crop: vec![],

            crop_length: None,
            crop_marker: None,

            attributes_to_highlight: vec![],
            highlight_pre_tag: None,
            highlight_post_tag: None,

            show_matches_position: None,

            sort: vec![],

            matching_strategy: None,
            show_ranking_score: None,
            attributes_to_search_on: vec![],
        }
    }

    pub fn hits_per_page(mut self, n: Option<u32>) -> Self {
        self.hits_per_page = n;
        self
    }

    pub fn page(mut self, p: Option<u32>) -> Self {
        self.page = p;
        self
    }

    pub fn sort_by(mut self, s: impl Into<String>) -> Self {
        self.sort.push(s.into());
        self
    }

    pub fn filter<S>(mut self, f: Option<S>) -> Self
    where
        String: From<S>,
    {
        self.filter = f.map(String::from);
        self
    }

    pub async fn search<T>(self, client: &crate::Client) -> crate::Result<SearchResponse<T>>
    where
        T: HasIndex,
        T: serde::de::DeserializeOwned,
    {
        T::search(client, self).await
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse<T> {
    /// Results of the query
    pub hits: Vec<T>,

    /// Number of documents skipped
    pub offset: u32,

    /// Number of documents to take
    pub limit: u32,

    /// Estimated total number of matches
    pub estimated_total_hits: Option<u32>,

    /// Exhaustive total number of matches
    pub total_hits: Option<u32>,

    /// Exhaustive total number of search result pages
    pub total_pages: Option<u32>,

    /// Number of results on each page
    pub hits_per_page: Option<u32>,

    /// Current search results page
    pub page: Option<u32>,

    /// Distribution of the given facets
    pub facet_distribution: Option<HashMap<String, HashMap<String, u32>>>,

    pub facet_stats: Option<HashMap<String, FacetStats>>,

    /// Processing time of the query
    pub processing_time_ms: u32,

    /// Query originating the response
    pub query: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct FacetStats {
    pub min: f32,
    pub max: f32,
}
