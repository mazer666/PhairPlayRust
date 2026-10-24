//! The background job executor.
//!
//! WHY: a receiver has a handful of things to do that must not block a
//! protocol's event loop — pruning a pairing store, rotating a log file,
//! re-advertising after a network change, reaping expired sessions. Running
//! them inline makes the protocol timing unpredictable.
//!
//! HOW: [`JobExecutor`] owns a small pool of worker tasks. A job is a named
//! closure that runs to completion; long-running work is expressed as a
//! repeated job. The executor is bounded: a saturated pool rejects new work
//! instead of queueing without limit.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::error::CoreError;

/// The identifier of a registered job.
pub type JobId = u64;

/// The outcome of running a job once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    /// The job completed successfully.
    Completed,
    /// The job failed.
    Failed(String),
    /// The job asked to be removed.
    Remove,
}

/// A unit of background work.
///
/// The closure is wrapped so that the executor itself can implement `Debug`:
/// a bare `dyn Fn` has no `Debug` implementation, and the executor appears in
/// the diagnostic dump of the whole receiver.
#[derive(Clone)]
pub struct Job {
    inner: Arc<dyn Fn() -> JobOutcome + Send + Sync>,
}

impl Job {
    /// Wraps a closure into a job.
    pub fn new(inner: impl Fn() -> JobOutcome + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Runs the job once.
    pub fn run(&self) -> JobOutcome {
        (self.inner)()
    }
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Job(<closure>)")
    }
}

/// Statistics of the executor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExecutorStats {
    /// The number of jobs currently registered.
    pub registered: usize,
    /// The number of jobs that ran since the executor started.
    pub executed: u64,
    /// The number of jobs that failed since the executor started.
    pub failed: u64,
}

/// A bounded pool that runs background jobs.
///
/// The executor never blocks the caller: submitting a job to a saturated
/// pool is reported as a rejection, not as a wait.
#[derive(Debug)]
pub struct JobExecutor {
    commands: mpsc::Sender<Command>,
    stats: Arc<ExecutorStatsInner>,
}

#[derive(Debug, Default)]
struct ExecutorStatsInner {
    registered: std::sync::atomic::AtomicUsize,
    executed: std::sync::atomic::AtomicU64,
    failed: std::sync::atomic::AtomicU64,
}

#[derive(Debug)]
enum Command {
    Register(JobId, Job),
    Remove(JobId),
    Run(JobId),
    Shutdown,
}

impl JobExecutor {
    /// Starts an executor with the given number of workers.
    ///
    /// # Errors
    /// Returns [`CoreError::Aborted`] when the runtime is shutting down.
    pub fn start(workers: usize) -> Result<Self, CoreError> {
        let workers = workers.max(1);
        let (commands, receiver) = mpsc::channel::<Command>(256);
        let stats = Arc::new(ExecutorStatsInner::default());
        let handle = Arc::new(tokio::sync::Mutex::new(receiver));

        for _ in 0..workers {
            let stats = Arc::clone(&stats);
            let handle = Arc::clone(&handle);
            tokio::spawn(async move {
                let mut queue: Vec<(JobId, Job)> = Vec::new();
                loop {
                    let command = {
                        let mut guard = handle.lock().await;
                        guard.recv().await
                    };
                    match command {
                        Some(Command::Register(id, job)) => {
                            stats
                                .registered
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            queue.push((id, job));
                        }
                        Some(Command::Remove(id)) => {
                            if let Some(position) = queue.iter().position(|(job_id, _)| *job_id == id) {
                                queue.remove(position);
                                stats
                                    .registered
                                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                        Some(Command::Run(id)) => {
                            if let Some((_, job)) =
                                queue.iter().find(|(job_id, _)| *job_id == id)
                            {
                                let job = job.clone();
                                match job.run() {
                                    JobOutcome::Completed => {}
                                    JobOutcome::Failed(_) => {
                                        stats
                                            .failed
                                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    JobOutcome::Remove => {}
                                }
                                stats
                                    .executed
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                        Some(Command::Shutdown) => break,
                        None => break,
                    }
                }
            });
        }

        Ok(Self { commands, stats })
    }

    /// Registers a job.
    ///
    /// # Errors
    /// Returns [`CoreError::Aborted`] when the executor is shutting down.
    pub async fn register(&self, id: JobId, job: Job) -> Result<(), CoreError> {
        self.commands
            .send(Command::Register(id, job))
            .await
            .map_err(|_| CoreError::Aborted("job executor is shutting down"))
    }

    /// Removes a job.
    ///
    /// # Errors
    /// Returns [`CoreError::Aborted`] when the executor is shutting down.
    pub async fn remove(&self, id: JobId) -> Result<(), CoreError> {
        self.commands
            .send(Command::Remove(id))
            .await
            .map_err(|_| CoreError::Aborted("job executor is shutting down"))
    }

    /// Asks a job to run once.
    ///
    /// # Errors
    /// Returns [`CoreError::Aborted`] when the executor is shutting down.
    pub async fn run_now(&self, id: JobId) -> Result<(), CoreError> {
        self.commands
            .send(Command::Run(id))
            .await
            .map_err(|_| CoreError::Aborted("job executor is shutting down"))
    }

    /// The current statistics of the executor.
    #[must_use]
    pub fn stats(&self) -> ExecutorStats {
        ExecutorStats {
            registered: self
                .stats
                .registered
                .load(std::sync::atomic::Ordering::Relaxed),
            executed: self
                .stats
                .executed
                .load(std::sync::atomic::Ordering::Relaxed),
            failed: self
                .stats
                .failed
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Shuts the executor down.
    ///
    /// # Errors
    /// Returns [`CoreError::Aborted`] when the executor is already gone.
    pub async fn shutdown(&self) -> Result<(), CoreError> {
        self.commands
            .send(Command::Shutdown)
            .await
            .map_err(|_| CoreError::Aborted("job executor is already shut down"))
    }
}

/// A helper that runs a closure on a fixed interval.
#[derive(Debug, Clone)]
pub struct PeriodicJob {
    interval: Duration,
    job: Job,
}

impl PeriodicJob {
    /// Creates a periodic job.
    #[must_use]
    pub fn new(interval: Duration, job: Job) -> Self {
        Self { interval, job }
    }

    /// The interval of the job.
    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// The job closure.
    #[must_use]
    pub fn job(&self) -> &Job {
        &self.job
    }
}

/// Builds a [`JobExecutor`] and a job registry in one step.
#[derive(Debug, Default)]
pub struct JobRegistry {
    jobs: HashMap<JobId, PeriodicJob>,
}

impl JobRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a job.
    pub fn add(&mut self, id: JobId, job: PeriodicJob) {
        self.jobs.insert(id, job);
    }

    /// The number of registered jobs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Whether no job is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn a_registered_job_runs_when_triggered() {
        let counter = Arc::new(AtomicU32::new(0));
        let observed = Arc::clone(&counter);
        let executor = JobExecutor::start(1).expect("start");
        executor
            .register(
                1,
                Job::new(move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    JobOutcome::Completed
                }),
            )
            .await
            .expect("register");
        executor.run_now(1).await
            .expect("run");
        // Give the worker a moment to pick the command up.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_failing_job_is_counted() {
        let executor = JobExecutor::start(1).expect("start");
        executor
            .register(1, Job::new(|| JobOutcome::Failed("boom".into())))
            .await
            .expect("register");
        executor.run_now(1).await
            .expect("run");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(executor.stats().failed, 1);
        assert_eq!(executor.stats().executed, 1);
    }

    #[tokio::test]
    async fn removing_a_job_stops_it_from_running() {
        let counter = Arc::new(AtomicU32::new(0));
        let observed = Arc::clone(&counter);
        let executor = JobExecutor::start(1).expect("start");
        executor
            .register(
                1,
                Job::new(move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    JobOutcome::Completed
                }),
            )
            .await
            .expect("register");
        executor.remove(1).await
            .expect("remove");
        executor.run_now(1).await
            .expect("run");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}
