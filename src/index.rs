use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub uid: String,
    pub created_at: String,
    pub updated_at: String,
    pub primary_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSettings {
    pub displayed_attributes: Vec<String>,
    pub searchable_attributes: Vec<String>,
    pub filterable_attributes: Vec<String>,
    pub sortable_attributes: Vec<String>,
    pub ranking_rules: Vec<String>,
    pub stop_words: Vec<String>,
    pub non_separator_tokens: Vec<String>,
    pub separator_tokens: Vec<String>,
    pub dictionary: Vec<String>,
    pub synonyms: HashMap<String, String>,
    pub distinct_attribute: serde_json::Value,
    pub typo_tolerance: TypoTolerance,
    pub faceting: Faceting,
    pub pagination: Pagination,
    pub proximity_precision: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypoTolerance {
    pub enabled: bool,
    pub min_word_size_for_typos: MinWordSizeForTypos,
    pub disable_on_words: Vec<String>,
    pub disable_on_attributes: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinWordSizeForTypos {
    pub one_typo: i64,
    pub two_typos: i64,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Faceting {
    pub max_values_per_facet: i64,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub max_total_hits: i64,
}
