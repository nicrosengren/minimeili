use crate::{AsTaskUid, Task};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use tokio::sync::{oneshot, Mutex};
use tracing::warn;

const MAX_TICKETS: usize = 512;

/// Manages pending tasks and notifies listeners on their
/// completion.
#[derive(Clone)]
pub struct TaskManager {
    task_tickets: Arc<Mutex<HashMap<u64, TaskTicket>>>,
}

pub enum TaskTicket {
    Pending(Vec<oneshot::Sender<Task>>),
    Completed(Task),
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
        let mut lock = self.task_tickets.lock().await;

        if MAX_TICKETS <= lock.len() {
            if let Some(min_key) = lock.keys().copied().min() {
                lock.remove(&min_key);
            }
        }

        match lock.entry(task.uid) {
            Entry::Vacant(vacant) => {
                vacant.insert(TaskTicket::Completed(task));
            }

            Entry::Occupied(mut entry) => match entry.get_mut() {
                TaskTicket::Completed(task) => {
                    warn!(
                        "received a hook for a task that already existed! ({})",
                        task.uid
                    );
                }

                TaskTicket::Pending(_) => {
                    let TaskTicket::Pending(waiters) =
                        entry.insert(TaskTicket::Completed(task.clone()))
                    else {
                        unreachable!("cannot occur since we check in the match clause");
                    };
                    println!("completing for {} waiters", waiters.len());

                    for sender in waiters {
                        if sender.send(task.clone()).is_err() {
                            warn!("listener went away");
                        }
                    }
                }
            },
        }
    }

    pub async fn wait_for_task(&self, task_uid: impl AsTaskUid) -> Option<Task> {
        let uid = task_uid.as_task_uid();
        let rx = {
            let mut lock = self.task_tickets.lock().await;

            match lock.entry(uid) {
                Entry::Occupied(ref mut occupied) => match occupied.get_mut() {
                    TaskTicket::Completed(task) => return Some(task.clone()),

                    TaskTicket::Pending(waiters) => {
                        let (tx, rx) = oneshot::channel();
                        waiters.push(tx);
                        rx
                    }
                },

                Entry::Vacant(vacant) => {
                    let mut v = Vec::with_capacity(4);
                    let (tx, rx) = oneshot::channel();
                    v.push(tx);
                    vacant.insert(TaskTicket::Pending(v));
                    rx
                }
            }
        };

        rx.await.ok()
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
            cloned_manager.wait_for_task(task_uid).await;
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
            manager1.wait_for_task(task_uid).await;
            task_uid
        });

        let manager2 = manager.clone();
        let handle2 = tokio::spawn(async move {
            manager2.wait_for_task(task_uid).await;
            task_uid
        });

        let manager3 = manager.clone();
        let handle3 = tokio::spawn(async move {
            manager3.wait_for_task(task_uid).await;
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
        manager.wait_for_task(task_uid).await;
    }

    #[tokio::test]
    async fn awaiting_notified_task() {
        let manager = TaskManager::default();

        let task_uid = 232;

        let manager1 = manager.clone();
        let handle1 = tokio::spawn(async move {
            manager1.wait_for_task(task_uid).await;
            task_uid
        });

        manager.handle_task(successful_task(task_uid)).await;

        assert!(handle1.await.is_ok());

        manager.wait_for_task(task_uid).await;
    }

    #[tokio::test]
    async fn multiple_waiters_on_multiple_tasks() {
        let manager = TaskManager::default();

        let mut handles = Vec::new();

        for i in 0..45 {
            let mc = manager.clone();
            let task_uid = (i % 3) as u64;

            let handle = tokio::spawn(async move {
                mc.wait_for_task(task_uid).await;
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
