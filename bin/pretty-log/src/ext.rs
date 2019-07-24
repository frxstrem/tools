pub trait OptionExt: private::OptionSealed {
    type Item;

    fn and_if<F>(self, f: F) -> Self
    where
        F: FnOnce() -> bool;
}

impl<T> OptionExt for Option<T> {
    type Item = T;

    fn and_if<F>(self, f: F) -> Option<T>
    where
        F: FnOnce() -> bool,
    {
        if self.is_some() && f() {
            self
        } else {
            None
        }
    }
}

mod private {
    pub trait OptionSealed {}
    impl<T> OptionSealed for Option<T> {}
}
