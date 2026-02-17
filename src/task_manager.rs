use crate::{AsTaskUid, Task};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use tokio::sync::{watch, Mutex};

const MAX_TICKETS: usize = 512;

#[derive(thiserror::Error, Debug)]
pub enum TaskPromiseError {
    #[error("Task manager closed unexpectedly")]
    ManagerClosed,

    #[error("waiting for task timed out after {ms}ms")]
    Timeout { ms: u128 },
}

/// Manages pending tasks and notifies listeners on their
/// completion.
#[derive(Clone)]
pub struct TaskManager {
    task_tickets: Arc<Mutex<HashMap<u64, TaskTicket>>>,
}

struct TaskTicket {
    channel: watch::Sender<Option<Task>>,
}

impl TaskTicket {
    fn pending() -> Self {
        let (channel, _) = watch::channel(None);
        Self { channel }
    }

    fn completed(task: Task) -> Self {
        let (channel, _) = watch::channel(Some(task));
        Self { channel }
    }

    fn complete(&mut self, task: Task) {
        let _ = self.channel.send(Some(task));
    }

    fn subscribe(&self) -> TaskPromise {
        TaskPromise {
            receiver: self.channel.subscribe(),
        }
    }
}

pub struct TaskPromise {
    receiver: watch::Receiver<Option<Task>>,
}

impl TaskPromise {
    pub async fn wait(mut self) -> Result<Task, TaskPromiseError> {
        let res = self
            .receiver
            .wait_for(|task_opt| task_opt.is_some())
            .await
            .map_err(|_| TaskPromiseError::ManagerClosed)?;

        Ok(res.as_ref().cloned().unwrap())
    }

    pub async fn wait_with_timeout(
        self,
        dur: tokio::time::Duration,
    ) -> Result<Task, TaskPromiseError> {
        match tokio::time::timeout(dur, self.wait()).await {
            Ok(Ok(task)) => Ok(task),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(TaskPromiseError::Timeout {
                ms: dur.as_millis(),
            }),
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            task_tickets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl TaskManager {
    pub async fn handle_task(&self, task: Task) {
        let mut tickets = self.task_tickets.lock().await;

        if MAX_TICKETS <= tickets.len() {
            if let Some(min_key) = tickets.keys().copied().min() {
                tickets.remove(&min_key);
            }
        }

        match tickets.entry(task.uid) {
            Entry::Vacant(vac) => {
                vac.insert(TaskTicket::completed(task));
            }

            Entry::Occupied(mut entry) => {
                entry.get_mut().complete(task);
            }
        }
    }

    pub async fn subscribe_for_task(&self, task_uid: impl AsTaskUid) -> TaskPromise {
        let uid = task_uid.as_task_uid();
        let mut tickets = self.task_tickets.lock().await;

        match tickets.entry(uid) {
            Entry::Occupied(occ) => occ.get().subscribe(),
            Entry::Vacant(vac) => vac.insert(TaskTicket::pending()).subscribe(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn single_waiter_on_pending_task() {
        let manager = TaskManager::default();

        let task_uid = 232;

        let cloned_manager = manager.clone();
        let waiter_handle = tokio::spawn(async move {
            cloned_manager.subscribe_for_task(task_uid).await;
            task_uid
        });

        manager.handle_task(successful_task(task_uid)).await;

        assert!(waiter_handle.await.is_ok());
    }

    #[tokio::test]
    async fn multiple_waiters_on_pending_task() {
        let manager = TaskManager::default();

        let task_uid = 232;

        let manager1 = manager.clone();
        let handle1 = tokio::spawn(async move {
            manager1.subscribe_for_task(task_uid).await;
            task_uid
        });

        let manager2 = manager.clone();
        let handle2 = tokio::spawn(async move {
            manager2.subscribe_for_task(task_uid).await;
            task_uid
        });

        let manager3 = manager.clone();
        let handle3 = tokio::spawn(async move {
            manager3.subscribe_for_task(task_uid).await;
            task_uid
        });

        manager.handle_task(successful_task(task_uid)).await;

        assert!(handle1.await.is_ok());
        assert!(handle2.await.is_ok());
        assert!(handle3.await.is_ok());
    }

    #[tokio::test]
    async fn single_waiter_on_finished_task() {
        let manager = TaskManager::default();

        let task_uid = 232;

        manager.handle_task(successful_task(task_uid)).await;
        manager.subscribe_for_task(task_uid).await;
    }

    #[tokio::test]
    async fn awaiting_notified_task() {
        let manager = TaskManager::default();

        let task_uid = 232;

        let manager1 = manager.clone();
        let handle1 = tokio::spawn(async move {
            manager1.subscribe_for_task(task_uid).await;
            task_uid
        });

        manager.handle_task(successful_task(task_uid)).await;

        assert!(handle1.await.is_ok());

        manager.subscribe_for_task(task_uid).await;
    }

    #[tokio::test]
    async fn multiple_waiters_on_multiple_tasks() {
        let manager = TaskManager::default();

        let mut handles = Vec::new();

        for i in 0..45 {
            let mc = manager.clone();
            let task_uid = (i % 3) as u64;

            let handle = tokio::spawn(async move {
                mc.subscribe_for_task(task_uid).await;
                task_uid
            });
            handles.push(handle);
        }

        for i in 0..3 {
            manager.handle_task(successful_task(i as u64)).await;
        }

        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    fn successful_task(uid: u64) -> Task {
        Task {
            uid,
            index_uid: String::from("afejkone"),
            status: crate::TaskStatus::Succeeded,
            kind: crate::TaskKind::IndexDeletion {
                deleted_documents: None,
            },
            canceled_by: None,
            error: None,
            duration: None,
            enqueued_at: String::from("fejkedtime"),
            started_at: None,
            finished_at: None,
        }
    }
}
