use std::any::Any;
use std::fmt::{self, Debug};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::thread::{self, JoinHandle};

use futures::channel::oneshot::{channel, Sender};
use futures::prelude::*;
use tokio::runtime::current_thread::{Handle, Runtime, TaskExecutor};

thread_local! {
    static CONTEXT: RefCell<Option<Box<dyn Any>>> = RefCell::new(None);
}

/// Tokio-based single-threaded runtime that runs tasks in a single background
/// thread.
pub struct BackgroundRuntime<C = ()>
where
    C: 'static + Send,
{
    handle: Handle,
    close_handle: Sender<()>,
    join_handle: JoinHandle<()>,
    _context: PhantomData<fn() -> C>,
}

impl BackgroundRuntime {
    /// Create a new `BackgroundRuntime` instance with an empty context, `()`.
    pub fn new() -> Result<BackgroundRuntime, SpawnError> {
        BackgroundRuntime::new_with_context(())
    }
}

impl<C> BackgroundRuntime<C>
where
    C: 'static + Send,
{
    /// Create a new `BackgroundRuntime` instance with a context value.
    #[allow(unused_must_use)]
    pub fn new_with_context(context: C) -> Result<BackgroundRuntime<C>, SpawnError> {
        // create a channel that will be used to send back the runtime handle
        let (tx, rx) = crate::channel::oneshot::channel();

        // create a future and handle that can be used to force the background thread to
        // stop as soon as it is idle
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

            // run tasks on the runtime until all spawned tasks are complete
            // and `close_handle` has been used or dropped
            runtime.spawn(async {
                // ignore result
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

    /// Run all tasks on the runtime to completion, and then stop the background thread.
    #[allow(unused_must_use)]
    pub fn stop(self) -> () {
        self.close_handle.send(());
        self.join_handle.join().unwrap();
    }

    /// Get the runtime context for the current thread.
    ///
    /// ## Panics
    ///
    /// This function will panic if called from outside a background thread
    /// belonging to a `BackgroundRuntime`.
    fn get_context() -> ContextRef<C> {
        CONTEXT.with(|context| {
            let context = context.borrow();
            let context = context
                .as_ref()
                .map(Box::as_ref)
                .and_then(Any::downcast_ref)
                .unwrap();
            unsafe {
                // NOTE: this is safe because `ContextRef` cannot be shared
                // across threads, and so it cannot outlive the thread local
                // storage
                ContextRef {
                    data: &*(context as *const C),
                }
            }
        })
    }

    /// Spawn a future onto the background thread runtime.
    pub fn spawn<F>(&self, future: F) -> Result<(), SpawnError>
    where
        F: 'static + Future<Output = ()> + Send,
    {
        self.handle.spawn(future).map_err(SpawnError::from)
    }

    /// Spawn a future from a function onto the background thread runtime.
    ///
    /// This method can be used to spawn futures on the background thread, even
    /// if the future is not `Send`, as long as `func` is `Send`.
    ///
    /// The function will be given a `ContextRef` value referencing the runtime
    /// context, which can be used to persist local thread storage across futures
    /// in the background thread.
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

    /// Spawn a future from a function onto the background thread runtime, and
    /// return a new future that is resolved when the future on the background
    /// thread has completed.
    pub async fn run<F, T>(&self, future: F) -> Result<T, SpawnError>
    where
        F: 'static + Future<Output = T> + Send,
        T: 'static + Send,
    {
        let (future, remote) = future.remote_handle();
        self.spawn(future)?;
        Ok(remote.await)
    }

    /// Spawn a future from a function onto the background thread runtime, and
    /// return a new future that is resolved when the future on the background
    /// thread has completed.
    ///
    /// This method can be used to spawn futures on the background thread, even
    /// if the future is not `Send`, as long as `func` is `Send`.
    ///
    /// The function will be given a `ContextRef` value referencing the runtime
    /// context, which can be used to persist local thread storage across futures
    /// in the background thread.
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

/// Reference to a [`BackgroundRuntime`](struct.BackgroundRuntime.html)'s context.
///
/// `ContextRef` does not implement `Send` or `Sync`, so it can only be used from
/// within the background thread of a `BackgroundRuntime`.
#[derive(Clone)]
pub struct ContextRef<C> {
    // NOTE: `data` is a pointer here because that forces `ContextRef` to be
    //       neither `Send` nor `Sync`
    data: *const C,
}

impl<C> Debug for ContextRef<C> where for<'a> &'a C : Debug {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ContextRef")
            .field("data", &self.deref())
            .finish()
    }
}

impl<C> Deref for ContextRef<C> {
    type Target = C;
    fn deref(&self) -> &C {
        unsafe { &*self.data }
    }
}

impl<C> AsRef<C> for ContextRef<C> {
    fn as_ref(&self) -> &C {
        &*self
    }
}

impl<C> Borrow<C> for ContextRef<C> {
    fn borrow(&self) -> &C {
        &*self
    }
}

impl<C> PartialEq<Self> for ContextRef<C>
where
    for<'a> &'a C: PartialEq<&'a C>,
{
    fn eq(&self, other: &Self) -> bool {
        <&C>::eq(&self.deref(), &other.deref())
    }
}

impl<C> Eq for ContextRef<C> where for<'a> &'a C: Eq {}

impl<C> PartialOrd<Self> for ContextRef<C>
where
    for<'a> &'a C: PartialOrd<&'a C>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <&'_ C>::partial_cmp(&self.deref(), &other.deref())
    }

    fn lt(&self, other: &Self) -> bool {
        <&'_ C>::lt(&self.deref(), &other.deref())
    }

    fn le(&self, other: &Self) -> bool {
        <&'_ C>::le(&self.deref(), &other.deref())
    }

    fn gt(&self, other: &Self) -> bool {
        <&'_ C>::gt(&self.deref(), &other.deref())
    }

    fn ge(&self, other: &Self) -> bool {
        <&'_ C>::ge(&self.deref(), &other.deref())
    }
}

impl<C> Ord for ContextRef<C>
where
    for<'a> &'a C: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        <&'_ C>::cmp(&self.deref(), &other.deref())
    }
}

impl<C> Hash for ContextRef<C> where for<'a> &'a C: Hash {
    fn hash<H>(&self, state: &mut H) where H : Hasher {
        self.deref().hash(state)
    }
}

/// Error occurred during spawning of a task on the `BackgroundRuntime`'s
/// background thread.
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
