//! Subagent module - Spawning of isolated subagents.
//!
//! Spawns an ISOLATED task execution (per REUSE-MAP: native isolation via gestalt-router
//! worktrees is Wave 2+; v1 = separate tokio::spawn with its own prompt context, documented).

use crate::ticker::RunTask;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

/// Spawner for executing isolated subagent runs.
///
/// Native isolation via gestalt-router worktrees is planned for Wave 2+.
/// In this version (v1), separate tokio::spawn is used with its own prompt context,
/// providing isolated execution contexts.
pub struct SubagentSpawner {
    /// Runner implementation.
    pub runner: Arc<dyn RunTask>,
    /// Limit on concurrent subagent tasks.
    pub max_concurrent: usize,
    /// Semaphore used to bound concurrency.
    pub semaphore: Arc<Semaphore>,
    /// Counter for generating unique subagent IDs.
    next_id: Arc<std::sync::atomic::AtomicUsize>,
}

/// Handle to a running subagent task.
pub struct SubagentHandle {
    /// Unique identifier for the subagent task.
    pub id: String,
    /// Join handle for the spawned tokio task.
    pub join: JoinHandle<Result<(), String>>,
}

impl SubagentSpawner {
    /// Creates a new SubagentSpawner with the given runner and concurrency limit.
    pub fn new(runner: Arc<dyn RunTask>, max_concurrent: usize) -> Self {
        Self {
            runner,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            next_id: Arc::new(std::sync::atomic::AtomicUsize::new(1)),
        }
    }

    /// Spawns an isolated subagent run.
    ///
    /// Subagent v1 = isolated tokio::spawn context; gestalt-router worktree isolation is the native follow-up.
    pub async fn spawn(&self, task: &str) -> Result<SubagentHandle, String> {
        let id_val = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let id = format!("subagent-{}", id_val);

        let runner = self.runner.clone();
        let semaphore = self.semaphore.clone();
        let task_owned = task.to_string();

        let join = tokio::spawn(async move {
            // Acquire permit from the semaphore to enforce concurrency limits
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|e| format!("Failed to acquire semaphore permit: {}", e))?;

            // Execute the isolated runner task
            runner.run(&task_owned).await
        });

        Ok(SubagentHandle { id, join })
    }
}

impl SubagentHandle {
    /// Awaits completion of the subagent run.
    pub async fn await_completion(self) -> Result<(), String> {
        match self.join.await {
            Ok(res) => res,
            Err(e) => Err(format!("Task panicked or was cancelled: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{sleep, Duration};

    struct TestRunner {
        current_concurrent: Arc<AtomicUsize>,
        peak_concurrent: Arc<AtomicUsize>,
        delay_ms: u64,
    }

    #[async_trait]
    impl RunTask for TestRunner {
        async fn run(&self, _task: &str) -> Result<(), String> {
            let prev = self.current_concurrent.fetch_add(1, Ordering::SeqCst);
            let current = prev + 1;
            loop {
                let peak = self.peak_concurrent.load(Ordering::SeqCst);
                if current > peak {
                    if self
                        .peak_concurrent
                        .compare_exchange(peak, current, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        break;
                    }
                } else {
                    break;
                }
            }

            sleep(Duration::from_millis(self.delay_ms)).await;

            self.current_concurrent.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_spawn_two_subagents() {
        let current_concurrent = Arc::new(AtomicUsize::new(0));
        let peak_concurrent = Arc::new(AtomicUsize::new(0));
        let runner = Arc::new(TestRunner {
            current_concurrent: current_concurrent.clone(),
            peak_concurrent: peak_concurrent.clone(),
            delay_ms: 30,
        });

        let spawner = SubagentSpawner::new(runner, 2);

        let h1 = spawner.spawn("task1").await.unwrap();
        let h2 = spawner.spawn("task2").await.unwrap();

        assert_ne!(h1.id, h2.id);

        let res1 = h1.await_completion().await;
        let res2 = h2.await_completion().await;

        assert!(res1.is_ok());
        assert!(res2.is_ok());
        assert!(peak_concurrent.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    async fn test_concurrency_limit_semaphore() {
        let current_concurrent = Arc::new(AtomicUsize::new(0));
        let peak_concurrent = Arc::new(AtomicUsize::new(0));
        let runner = Arc::new(TestRunner {
            current_concurrent: current_concurrent.clone(),
            peak_concurrent: peak_concurrent.clone(),
            delay_ms: 30,
        });

        // Max 2 concurrent tasks, but we will spawn 3.
        let spawner = SubagentSpawner::new(runner, 2);

        let h1 = spawner.spawn("task1").await.unwrap();
        let h2 = spawner.spawn("task2").await.unwrap();
        let h3 = spawner.spawn("task3").await.unwrap();

        let _ = tokio::join!(
            h1.await_completion(),
            h2.await_completion(),
            h3.await_completion()
        );

        // Max concurrent should be exactly 2
        let max_seen = peak_concurrent.load(Ordering::SeqCst);
        assert!(max_seen <= 2, "Peak concurrency exceeded max limit: {}", max_seen);
        assert_eq!(current_concurrent.load(Ordering::SeqCst), 0);
    }
}
