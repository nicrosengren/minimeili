use crate::{DateTime, IndexSettings};

const DEFAULT_TASK_WAIT_DUR: std::time::Duration = std::time::Duration::from_millis(500);

pub trait AsTaskUid {
    fn as_task_uid(&self) -> u64;
}

impl AsTaskUid for u64 {
    fn as_task_uid(&self) -> u64 {
        *self
    }
}

impl AsTaskUid for &'_ TaskRef {
    fn as_task_uid(&self) -> u64 {
        self.uid
    }
}

impl AsTaskUid for &'_ Task {
    fn as_task_uid(&self) -> u64 {
        self.uid
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRef {
    #[serde(rename = "taskUid")]
    pub uid: u64,
    pub index_uid: String,
    pub status: TaskStatus,

    #[serde(rename = "type")]
    pub kind: TaskKindRef,

    pub enqueued_at: DateTime,
}

impl TaskRef {
    #[cfg(feature = "tokio")]
    pub async fn wait_until_stopped(&self, c: &crate::Client) -> crate::Result<Task> {
        c.wait_for_task(self, DEFAULT_TASK_WAIT_DUR).await
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub uid: u64,
    pub index_uid: String,
    pub status: TaskStatus,
    #[serde(flatten)]
    pub kind: TaskKind,
    pub canceled_by: Option<u64>,
    pub error: Option<TaskError>,
    pub duration: Option<String>,
    pub enqueued_at: DateTime,
    pub started_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
}

impl Task {
    #[cfg(feature = "tokio")]
    pub async fn wait_until_stopped(&self, c: &crate::Client) -> crate::Result<Task> {
        c.wait_for_task(self, DEFAULT_TASK_WAIT_DUR).await
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
    pub message: String,
    pub code: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub link: String,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Enqueued,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

impl TaskStatus {
    pub fn has_stopped(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "details")]
pub enum TaskKind {
    #[serde(rename_all = "camelCase")]
    IndexCreation {
        primary_key: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    IndexUpdate {
        primary_key: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    IndexDeletion {
        deleted_documents: Option<i64>,
    },
    #[serde(rename_all = "camelCase")]
    IndexSwap {
        swaps: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    DocumentAdditionOrUpdate {
        received_documents: u64,
        indexed_documents: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    DocumentDeletion {
        provided_ids: u64,
        original_filter: Option<String>,
        deleted_documents: u64,
    },

    SettingsUpdate(IndexSettings),

    #[serde(rename_all = "camelCase")]
    DumpCreation {
        dump_uid: i64,
    },
    #[serde(rename_all = "camelCase")]
    TaskCancelation {
        /// The number of matched tasks. If the API key used for the request doesn’t
        /// have access to an index, tasks relating to that index will not be included
        /// in matchedTasks
        matched_tasks: i64,
        /// The number of tasks successfully canceled.
        /// If the task cancelation fails, this will be 0.
        /// null when the task status is enqueued or processing
        canceled_tasks: i64,

        /// The filter used in the cancel task request
        original_filter: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    TaskDeletion {
        /// The number of matched tasks. If the API key used for the request
        /// doesn’t have access to an index, tasks relating to that index will
        /// not be included in matchedTasks
        matched_tasks: u64,
        /// The number of tasks successfully deleted. If the task deletion fails,
        /// this will be 0. null when the task status is enqueued or processing
        deleted_tasks: u64,

        /// The filter used in the delete task request
        original_filter: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    SnapshotCreation,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskKindRef {
    IndexCreation,
    IndexUpdate,
    IndexDeletion,
    IndexSwap,
    DocumentAdditionOrUpdate,
    DocumentDeletion,
    SettingsUpdate,
    DumpCreation,
    TaskCancelation,
    TaskDeletion,

    SnapshotCreation,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn deserialize_task_kind_document_addition() {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct St {
            #[serde(flatten)]
            kind: TaskKind,
        }

        println!(
            "Serialized:\n{}",
            serde_json::to_string(&St {
                kind: TaskKind::DocumentAdditionOrUpdate {
                    received_documents: 1,
                    indexed_documents: Some(23),
                }
            })
            .expect("ser")
        );

        let s = r#"
{"type":"documentAdditionOrUpdate","details":{"receivedDocuments":1,"indexedDocuments":null}} "#;

        let res = serde_json::from_str::<St>(s).expect("deser");

        assert!(matches!(
            res.kind,
            TaskKind::DocumentAdditionOrUpdate { .. }
        ))
    }
}
