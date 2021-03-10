#![no_std]

use cty::c_char;

/// A status code the represents the outcome of a Rust-side function,
/// intended to be sent back to GameMaker.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OutputCode(f64);
impl OutputCode {
    /// Represents an operation that executed as intended.
    pub const SUCCESS: OutputCode = OutputCode(1.0);
    /// Represents an operation that failed to execute as intended.
    pub const FAILURE: OutputCode = OutputCode(0.0);
}

/// Representation of a pointer sent from GameMaker. Dereferences
/// into its inner c_char.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GmPtr(*const c_char);
impl GmPtr {
    /// Creates a new GmPtr based on the given pointer.
    pub fn new(ptr: *const c_char) -> Self {
        Self(ptr)
    }

    /// Returns a copy of the inner value.
    pub fn inner(&self) -> *const c_char {
        self.0
    }

    /// Transforms the inner value into an &str.
    ///
    /// # Saftey
    /// Assumes that the pointer being used is valid as a c_str pointer.
    pub fn to_str(self) -> Result<&'static str, core::str::Utf8Error> {
        unsafe { cstr_core::CStr::from_ptr(self.0) }.to_str()
    }
}
impl core::ops::Deref for GmPtr {
    type Target = *const c_char;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This is a Gm Id for a buffer, or any other dynamically allocated texture.
/// It is transparent and can be sent back to Gm as a Parameter or as a Return type.
///
/// Generally, you shouldn't be constructing this, but should be getting this from Gm.
/// The one exception is in Unit Tests, where you can get access to a `new` method, or
/// the `dummy` variant, which will give you an f64::MAX inside.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct GmId(f64);

impl GmId {
    /// Creates a new ID. This is intended for Units Tests.
    #[cfg(test)]
    pub fn new(id: f64) -> Self {
        Self(id)
    }

    /// Returns a dummy, with the f64::MAX inside it.
    pub fn dummy() -> Self {
        Self(f64::MAX)
    }
}

/// Our basic GmBuffer. This holds anything you want.
///
/// # Safety
/// We LIE to Rust and tell it that the buffer held within is `'static`.
/// It is **not** static, but we're going to act like it is. Because GM is our
/// allocator, a user could easily decide to deallocate a buffer.
///
/// We would very much so like if they don't do that, and will pretend like they cannot.
/// If, however, they do, this entire data structure will be inadequate.
#[derive(Debug)]
pub struct GmBuffer<T: 'static> {
    /// An Id for the GameMaker buffer to return when we want to destruct this.
    id: GmId,

    /// The actual vertex buffer that we write to.
    pub buffer: &'static mut [T],
}

impl<T> GmBuffer<T> {
    /// Creates a new Gm Buffer.
    ///
    /// - `gm_id` is the id, in GameMaker, of the buffer we're trying to create.
    /// - `gm_ptr` is the pointer provided to the buffer that GameMaker gives us.
    /// - `len` is the number of T's that can be fit within the buffer, **not** the
    /// number of bytes. For more information, see [from_raw_parts](core::slice::from_raw_parts_mut)
    ///
    /// # Safety
    /// Buffer must be allocated BY GAMEMAKER, not by some Rust code. The following invariants, in particular
    /// must be held in order for this type to be safe:
    /// - The buffer must be valid until `GmBuffer` is dropped
    /// - The buffer's `id` must be a valid `GmId` from GameMaker.
    /// - T must be sized, non-zero sized, and **must be zeroable**. This means that an "all zeroes"
    ///   representation of the buffer is valid.  
    pub unsafe fn new(gm_id: GmId, gm_ptr: GmPtr, len: usize) -> Self {
        let buffer = {
            let buf = gm_ptr.inner() as *mut c_char as *mut T;

            core::slice::from_raw_parts_mut(buf, len)
        };

        Self { id: gm_id, buffer }
    }

    /// This destructs the Buffer, taking self, and returning the Id. Once we give up ownership
    /// of the ID by exposing it, we assume that we cannot safely hold onto the buffer anymore (ie,
    /// we assume that it will be destroyed), and therefore, this function takes `self`.
    pub fn id(self) -> GmId {
        self.id
    }
}

impl<T> core::ops::Index<usize> for GmBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl<T> core::ops::IndexMut<usize> for GmBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_null_ptr() {
        GmPtr::new(core::ptr::null());
    }

    #[test]
    fn make_string_ptr() {
        GmPtr::new("Hello, world!\0".as_ptr() as *const cty::c_char);
    }

    #[test]
    fn read_string_ptr() {
        let ptr = GmPtr::new("Hello, world!\0".as_ptr() as *const cty::c_char);
        let out = ptr.to_str().unwrap();
        assert_eq!(out, "Hello, world!");
    }
}
