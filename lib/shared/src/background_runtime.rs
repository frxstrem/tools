use std::thread::{self, JoinHandle};

use futures::channel::oneshot::{channel, Sender};
use futures::prelude::*;
use tokio::runtime::current_thread::{Handle, Runtime, TaskExecutor};

pub struct BackgroundRuntime {
    handle: Handle,
    close_handle: Sender<()>,
    join_handle: JoinHandle<()>,
}

impl BackgroundRuntime {
    #[allow(unused_must_use)]
    pub fn new() -> Result<BackgroundRuntime, SpawnError> {
        let (tx, rx) = crate::channel::oneshot::channel();

        let (close_handle, closed) = channel();

        // spawn background thread for tasks
        let join_handle = thread::spawn(move || {
            // create single-threaded runtime, and get handle to runtime
            let result = Runtime::new()
                .map(|runtime| {
                    let handle = runtime.handle();
                    (runtime, handle)
                })
                .map_err(|_| ());

            let (mut runtime, handle) = match result {
                Ok(value) => value,
                Err(err) => {
                    tx.send(Err(err)).unwrap();
                    return;
                }
            };

            tx.send(Ok(handle));

            runtime.spawn(async {
                closed.await;
            });
            runtime.run();
        });

        // wait for handle from background thread
        let handle = match rx.recv() {
            Ok(Ok(handle)) => handle,
            Ok(Err(_)) | Err(_) => return Err(SpawnError::BackgroundThreadNotStarted),
        };

        // construct runtime object
        Ok(BackgroundRuntime {
            handle,
            close_handle,
            join_handle,
        })
    }

    #[allow(unused_must_use)]
    pub fn stop(self) -> () {
        self.close_handle.send(());
        self.join_handle.join().unwrap();
    }

    pub fn spawn<F>(&self, future: F) -> Result<(), SpawnError>
    where
        F: 'static + Future<Output = ()> + Send,
    {
        self.handle.spawn(future).map_err(SpawnError::from)
    }

    pub fn spawn_with<F, G>(&self, func: G) -> Result<(), SpawnError>
    where
        G: 'static + FnOnce() -> F + Send,
        F: 'static + Future<Output = ()>,
    {
        self.spawn(async move {
            let future = Box::pin(func());
            TaskExecutor::current().spawn_local(future).unwrap();
        })
    }

    pub async fn run<F, T>(&self, future: F) -> Result<T, SpawnError>
    where
        F: 'static + Future<Output = T> + Send,
        T: 'static + Send,
    {
        let (future, remote) = future.remote_handle();
        self.spawn(future)?;
        Ok(remote.await)
    }

    #[allow(unused_must_use)]
    pub async fn run_with<F, G, T>(&self, func: G) -> Result<T, SpawnError>
    where
        G: 'static + FnOnce() -> F + Send,
        F: 'static + Future<Output = T>,
        T: 'static + Send,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.spawn(async move {
            let func = move || {
                async move {
                    let result = func().await;
                    tx.send(result);
                }
            };
            let future = Box::pin(func());
            TaskExecutor::current().spawn_local(future).unwrap();
        })?;

        rx.await.map_err(SpawnError::from)
    }
}

#[derive(Debug)]
pub enum SpawnError {
    BackgroundThreadNotStarted,
    TokioSpawnError(tokio::executor::SpawnError),
    Canceled,
}

impl From<tokio::executor::SpawnError> for SpawnError {
    fn from(err: tokio::executor::SpawnError) -> SpawnError {
        SpawnError::TokioSpawnError(err)
    }
}

impl From<futures::channel::oneshot::Canceled> for SpawnError {
    fn from(_err: futures::channel::oneshot::Canceled) -> SpawnError {
        SpawnError::Canceled
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::timer::delay_for;

    #[test]
    pub fn background_runtime() {
        let runtime = BackgroundRuntime::new().unwrap();

        let mut results: Vec<Arc<AtomicBool>> = Vec::new();

        {
            let out = Arc::new(AtomicBool::new(false));
            results.push(out.clone());

            runtime
                .spawn(async move {
                    delay_for(Duration::from_millis(300)).await;
                    out.store(true, Ordering::Relaxed);
                })
                .unwrap();
        }

        {
            let out = Arc::new(AtomicBool::new(false));
            results.push(out.clone());

            runtime
                .spawn_with(move || {
                    async move {
                        delay_for(Duration::from_millis(400)).await;
                        out.store(true, Ordering::Relaxed);
                    }
                })
                .unwrap();
        }

        runtime.stop();

        for result in results {
            assert!(result.load(Ordering::Relaxed));
        }
    }
}
