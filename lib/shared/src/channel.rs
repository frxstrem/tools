

pub mod oneshot {
    use std::sync::{Arc, Mutex, Condvar};

    pub struct Sender<T> {
        pair: Arc<(Mutex<Option<Result<T, Cancelled>>>, Condvar)>,
    }

    impl<T> Sender<T> {
        pub fn send(self, t: T) -> Result<(), T> {
            let (m, cv) = &*self.pair;
            m.lock().unwrap().get_or_insert(Ok(t));
            cv.notify_one();
            Ok(())
        }
    }

    impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
            let (m, cv) = &*self.pair;
            m.lock().unwrap().get_or_insert(Err(Cancelled));
            cv.notify_one();
        }
    }

    pub struct Receiver<T> {
        pair: Arc<(Mutex<Option<Result<T, Cancelled>>>, Condvar)>,
    }

    impl<T> Receiver<T> {
        pub fn recv(self) -> Result<T, Cancelled> {
            let (m, cv) = &*self.pair;
            let mut lock = m.lock().unwrap();
            loop {
                match lock.take() {
                    Some(value) => break value,
                    None => lock = cv.wait(lock).unwrap(),
                }
            }
        }
    }

    pub struct Cancelled;

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let pair = Arc::new((Mutex::new(None), Condvar::new()));
        (Sender { pair: pair.clone() }, Receiver { pair })
    }
}
