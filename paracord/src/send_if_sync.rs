use std::{mem::ManuallyDrop, ops::Deref};

/// A type that prevents mut access to `T`, which allows it to be `Send`` if `T` is `Sync``.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SendIfSync<T>(ManuallyDrop<T>);

// Safety: `SendIfSync` forbids any mut access to T.
unsafe impl<T: Sync> Send for SendIfSync<T> {}

impl<T> SendIfSync<T> {
    pub fn cast_to_slice(this: &[Self]) -> &[T] {
        // Safety: `SendIfSync` is transparent to T
        unsafe { std::mem::transmute(this) }
    }

    pub fn cast_from_slice(this: &[T]) -> &[Self] {
        // Safety: `SendIfSync` is transparent to T
        unsafe { std::mem::transmute(this) }
    }
}

impl<T> Deref for SendIfSync<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
