//! Ticker logic for periodic task execution (interval-based cron v1).
//!
//! Note on Cron parsing:
//! Real cron-syntax parsing (e.g. "* * * * *") will be integrated in future versions.
//! Currently, the scheduler operates in an INTERVAL-BASED (seconds) mode.
//! `cron_expr` acts as the interval duration in seconds (e.g. "5" for 5 seconds),
//! or with "ms" suffix for finer control in unit testing (e.g. "50ms").

use async_trait::async_trait;
use std::sync::Arc;

/// A trait for executing scheduled tasks.
///
/// Implemented by AgentLoop wrappers or test mocks.
#[async_trait]
pub trait RunTask: Send + Sync {
    /// Executes the given task prompt.
    async fn run(&self, task: &str) -> Result<(), String>;
}

/// A scheduled task entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTask {
    /// Unique name of the task
    pub name: String,
    /// Interval-based string or cron syntax in the future
    pub cron_expr: String,
    /// Prompt text to pass to RunTask
    pub prompt: String,
}

/// A periodic tasks scheduler.
pub struct Scheduler {
    /// List of scheduled tasks
    pub tasks: Vec<ScheduledTask>,
    /// Task runner implementation
    pub runner: Arc<dyn RunTask>,
}

impl Scheduler {
    /// Creates a new Scheduler.
    pub fn new(runner: Arc<dyn RunTask>) -> Self {
        Self {
            tasks: Vec::new(),
            runner,
        }
    }

    /// Adds a task to the scheduler with a specified interval in seconds.
    pub fn add_task(&mut self, name: String, interval_secs: u64, prompt: String) {
        self.tasks.push(ScheduledTask {
            name,
            cron_expr: interval_secs.to_string(),
            prompt,
        });
    }

    /// Loops over all registered tasks and spawns background tasks
    /// that sleep for the configured interval and execute the task's prompt.
    pub async fn run_forever(&self) {
        let mut handles = Vec::new();

        for task in &self.tasks {
            let runner = self.runner.clone();
            let prompt = task.prompt.clone();
            let name = task.name.clone();

            // Parse interval from cron_expr. Supports decimal float seconds,
            // or milliseconds if it has "ms" suffix.
            let duration = if task.cron_expr.ends_with("ms") {
                let ms: u64 = task.cron_expr.trim_end_matches("ms").parse().unwrap_or(1000);
                tokio::time::Duration::from_millis(ms)
            } else if let Ok(secs) = task.cron_expr.parse::<f64>() {
                tokio::time::Duration::from_secs_f64(secs)
            } else {
                tokio::time::Duration::from_secs(1)
            };

            let handle = tokio::spawn(async move {
                loop {
                    tokio::time::sleep(duration).await;
                    tracing::info!("Scheduled task '{}' firing", name);
                    if let Err(e) = runner.run(&prompt).await {
                        tracing::error!("Error executing scheduled task '{}': {}", name, e);
                    }
                }
            });
            handles.push(handle);
        }

        if handles.is_empty() {
            // Keep the task alive indefinitely if no tasks are registered.
            std::future::pending::<()>().await;
        } else {
            for handle in handles {
                let _ = handle.await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockRunner {
        counter: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl RunTask for MockRunner {
        async fn run(&self, task: &str) -> Result<(), String> {
            assert_eq!(task, "test-prompt");
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    async fn assert_counter_eventually(counter: &Arc<AtomicUsize>, expected: usize) {
        for _ in 0..100 {
            if counter.load(Ordering::SeqCst) == expected {
                return;
            }
            tokio::task::yield_now().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        panic!(
            "Counter did not reach {} in time. Actual: {}",
            expected,
            counter.load(Ordering::SeqCst)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_scheduler_ticker() {
        let counter = Arc::new(AtomicUsize::new(0));
        let runner = Arc::new(MockRunner {
            counter: counter.clone(),
        });

        let mut scheduler = Scheduler::new(runner);

        // Test with 50ms interval
        scheduler.tasks.push(ScheduledTask {
            name: "test-task".to_string(),
            cron_expr: "50ms".to_string(),
            prompt: "test-prompt".to_string(),
        });

        let handle = tokio::spawn(async move {
            scheduler.run_forever().await;
        });

        // Let the background task start and execute up to its first sleep
        tokio::task::yield_now().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        // Initially counter should be 0
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // Advance time by 50ms and assert
        tokio::time::advance(tokio::time::Duration::from_millis(50)).await;
        assert_counter_eventually(&counter, 1).await;

        // Advance time by another 50ms (total 100ms)
        tokio::time::advance(tokio::time::Duration::from_millis(50)).await;
        assert_counter_eventually(&counter, 2).await;

        // Advance time by another 100ms (total 200ms)
        tokio::time::advance(tokio::time::Duration::from_millis(100)).await;
        assert_counter_eventually(&counter, 4).await;

        handle.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn test_scheduler_add_task_seconds() {
        let counter = Arc::new(AtomicUsize::new(0));
        let runner = Arc::new(MockRunner {
            counter: counter.clone(),
        });

        let mut scheduler = Scheduler::new(runner);
        scheduler.add_task("seconds-task".to_string(), 1, "test-prompt".to_string());

        assert_eq!(scheduler.tasks.len(), 1);
        assert_eq!(scheduler.tasks[0].cron_expr, "1");

        let handle = tokio::spawn(async move {
            scheduler.run_forever().await;
        });

        // Let the background task start and execute up to its first sleep
        tokio::task::yield_now().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // Advance by 1 second
        tokio::time::advance(tokio::time::Duration::from_secs(1)).await;
        assert_counter_eventually(&counter, 1).await;

        // Advance by another 1 second
        tokio::time::advance(tokio::time::Duration::from_secs(1)).await;
        assert_counter_eventually(&counter, 2).await;

        handle.abort();
    }
}
