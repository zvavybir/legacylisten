// All these functions are un-idiomatic, but I don't know how to fix
// it.

use std::{sync::mpsc::Receiver, thread, time::Duration};

pub fn recv_last<T>(rx: &Receiver<T>) -> T
{
    let mut rv = None;

    loop
    {
        while let Ok(v) = rx.try_recv()
        {
            rv = Some(v);
        }

        if let Some(v) = rv
        {
            break v;
        }

        thread::sleep(Duration::from_micros(1));
    }
}

pub fn take_error<T: Clone, E>(x: Result<T, E>) -> (Result<T, E>, Option<T>)
{
    match x
    {
        Ok(v) => (Ok(v.clone()), Some(v)),
        Err(e) => (Err(e), None),
    }
}

pub fn unwrap_two<T, U, F>(x: Option<T>, y: Option<U>, f: F) -> (T, U)
where
    F: FnOnce() -> (T, U),
{
    match (x, y)
    {
        (Some(x), Some(y)) => (x, y),
        (Some(x), None) => (x, f().1),
        (None, Some(y)) => (f().0, y),
        (None, None) => f(),
    }
}

pub trait ResultExtend
{
    type Inner;

    fn flatten_stable(self) -> Self::Inner;
}

// Unbelievably this is not stable yet.  TODO: Remove when
// https://github.com/rust-lang/rust/issues/70142 becomes stable.
impl<T, E> ResultExtend for Result<Result<T, E>, E>
{
    type Inner = Result<T, E>;

    fn flatten_stable(self) -> Self::Inner
    {
        match self
        {
            Ok(x) => x,
            Err(e) => Err(e),
        }
    }
}

impl<T, E> ResultExtend for Result<T, Result<T, E>>
{
    type Inner = Result<T, E>;

    fn flatten_stable(self) -> Self::Inner
    {
        match self
        {
            Ok(x) => Ok(x),
            Err(e) => e,
        }
    }
}
