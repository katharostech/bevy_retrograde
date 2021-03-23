//! This module was extracted from GFX:
//!
//! It is under the following license:
//! https://github.com/gfx-rs/gfx/blob/843c68494fa4fe457d145bf53e09bb7d25bb8aa2/src/backend/gl/src/lib.rs#L266

#![allow(dead_code)]

use std::{
    fmt,
    sync::{Arc, Weak},
    thread,
};

/// Single-threaded `Arc`.
/// Wrapper for `Arc` that allows you to `Send` it even if `T: !Sync`.
/// Yet internal data cannot be accessed outside of the thread where it was created.
pub(crate) struct Starc<T: ?Sized> {
    arc: Arc<T>,
    thread: thread::ThreadId,
}

impl<T: ?Sized> Clone for Starc<T> {
    fn clone(&self) -> Self {
        Self {
            arc: self.arc.clone(),
            thread: self.thread,
        }
    }
}

impl<T: ?Sized> fmt::Debug for Starc<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:p}@{:?}", self.arc, self.thread)
    }
}

impl<T> Starc<T> {
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Starc {
            arc: Arc::new(value),
            thread: thread::current().id(),
        }
    }
    #[inline]
    pub(crate) fn try_unwrap(self) -> Result<T, Self> {
        let a = Arc::try_unwrap(self.arc);
        let thread = self.thread;
        a.map_err(|a| Starc {
            arc: a,
            thread: thread,
        })
    }
}

impl<T> Starc<T>
where
    T: ?Sized,
{
    #[inline]
    pub(crate) fn downgrade(this: &Starc<T>) -> Wstarc<T> {
        Wstarc {
            weak: Arc::downgrade(&this.arc),
            thread: this.thread,
        }
    }

    #[inline]
    pub(crate) fn get_mut(this: &mut Starc<T>) -> Option<&mut T> {
        Arc::get_mut(&mut this.arc)
    }
}

unsafe impl<T: ?Sized> Send for Starc<T> {}
unsafe impl<T: ?Sized> Sync for Starc<T> {}

impl<T: ?Sized> std::ops::Deref for Starc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        assert_eq!(thread::current().id(), self.thread);
        &*self.arc
    }
}

/// Single-threaded `Weak`.
/// Wrapper for `Weak` that allows you to `Send` it even if `T: !Sync`.
/// Yet internal data cannot be accessed outside of the thread where it was created.
pub(crate) struct Wstarc<T: ?Sized> {
    weak: Weak<T>,
    thread: thread::ThreadId,
}
impl<T> Wstarc<T> {
    pub(crate) fn upgrade(&self) -> Option<Starc<T>> {
        let thread = self.thread;
        self.weak.upgrade().map(|arc| Starc { arc, thread })
    }
}
unsafe impl<T: ?Sized> Send for Wstarc<T> {}
unsafe impl<T: ?Sized> Sync for Wstarc<T> {}
