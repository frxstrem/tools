use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::thread::{self, JoinHandle};

use futures::channel::oneshot::{channel, Sender};
use futures::prelude::*;
use tokio::runtime::current_thread::{Handle, Runtime, TaskExecutor};

thread_local! {
    static CONTEXT: RefCell<Option<Box<dyn Any>>> = RefCell::new(None);
}

pub struct BackgroundRuntime<C = ()>
where
    C: 'static + Send,
{
    handle: Handle,
    close_handle: Sender<()>,
    join_handle: JoinHandle<()>,
    _context: PhantomData<fn() -> C>,
}

impl BackgroundRuntime<> {
    pub fn new() -> Result<BackgroundRuntime<>,SpawnError> {
        BackgroundRuntime::new_with_context(())
    }
}

impl<C> BackgroundRuntime<C>
where
    C: 'static + Send,
{
    #[allow(unused_must_use)]
    pub fn new_with_context(context: C) -> Result<BackgroundRuntime<C>, SpawnError> {
        let (tx, rx) = crate::channel::oneshot::channel();

        let (close_handle, closed) = channel();

        // spawn background thread for tasks
        let join_handle = thread::spawn(move || {
            CONTEXT.with(|cell| {
                *cell.borrow_mut() = Some(Box::new(context));
            });

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
            _context: PhantomData,
        })
    }

    #[allow(unused_must_use)]
    pub fn stop(self) -> () {
        self.close_handle.send(());
        self.join_handle.join().unwrap();
    }

    fn get_context() -> ContextRef<C> {
        CONTEXT.with(|context| {
            let context = context.borrow();
            let context = context
                .as_ref()
                .map(Box::as_ref)
                .and_then(Any::downcast_ref)
                .unwrap();
            unsafe {
                ContextRef {
                    data: &*(context as *const C),
                }
            }
        })
    }

    pub fn spawn<F>(&self, future: F) -> Result<(), SpawnError>
    where
        F: 'static + Future<Output = ()> + Send,
    {
        self.handle.spawn(future).map_err(SpawnError::from)
    }

    pub fn spawn_with<F, G>(&self, func: G) -> Result<(), SpawnError>
    where
        G: 'static + FnOnce(ContextRef<C>) -> F + Send,
        F: 'static + Future<Output = ()>,
    {
        self.spawn(async move {
            let state = Self::get_context();
            let future = Box::pin(func(state));
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
        G: 'static + FnOnce(ContextRef<C>) -> F + Send,
        F: 'static + Future<Output = T>,
        T: 'static + Send,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.spawn(async move {
            let func = move || {
                async move {
                    let context = Self::get_context();
                    let result = func(context).await;
                    tx.send(result);
                }
            };
            let future = Box::pin(func());
            TaskExecutor::current().spawn_local(future).unwrap();
        })?;

        rx.await.map_err(SpawnError::from)
    }
}

pub struct ContextRef<S> {
    data: *const S,
}

impl<S> Deref for ContextRef<S> {
    type Target = S;
    fn deref(&self) -> &S {
        unsafe { &*self.data }
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
        let context = 123;

        let runtime = BackgroundRuntime::new_with_context(context).unwrap();

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
                .spawn_with(move |ctx| {
                    async move {
                        delay_for(Duration::from_millis(400)).await;

                        assert_eq!(&*ctx, &123);

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
