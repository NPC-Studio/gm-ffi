//! A Rust crate to interface between GameMaker and Rust.

#![cfg_attr(not(test), no_std)] // this means we are std in test
#![cfg_attr(test, allow(clippy::float_cmp))] // lets us compare floats in asserts
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![deny(missing_docs)]
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

    /// Creates a custom OutputCode. This can mean whatever you want it to mean,
    /// for example, returning the number of bytes written into a shared buffer.
    pub fn custom(code: f64) -> Self {
        Self(code)
    }
}

// blanket implementation
impl<T, E> From<Result<T, E>> for OutputCode {
    fn from(o: Result<T, E>) -> Self {
        if o.is_ok() {
            OutputCode::SUCCESS
        } else {
            OutputCode::FAILURE
        }
    }
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

    /// Returns a self with `NULL` inside it. Be careful out there!
    ///
    /// # Safety
    /// It's a nullptr, come on you dummy! You can obviously break everything
    /// with this.
    pub unsafe fn null() -> Self {
        Self(core::ptr::null())
    }

    /// Returns a copy of the inner value.
    pub fn inner(self) -> *const c_char {
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

/// This is a Gm Id for a buffer, or any other dynamically allocated resource.
/// It is transparent in memory but opaque in type (ie, you can't inspect what's inside it),
/// so it can be sent back and forth to GM as an f64.
///
/// If you want to inspect an ID from Gm, you probably want `GmResourceId`, which is transparent
/// in type as well.
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
    pub const fn new(id: f64) -> Self {
        Self(id)
    }

    /// Returns a dummy, with the f64::MAX inside it.
    pub const fn dummy() -> Self {
        Self(f64::MAX)
    }
}

/// This is a Gm Real, which can be a resource, or any other stable resource.
///
/// Generally, you shouldn't be constructing this, but should be getting this from Gm.
/// The one exception is in Unit Tests, where you can get access to a `new` method, or
/// the `dummy` variant, which will give you an f64::MAX inside.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct GmReal(pub f64);

impl GmReal {
    /// Creates a new ID. You probably shouldn't be using this, but there are times
    /// when you might be reconstructing it.
    pub const fn new(id: f64) -> Self {
        Self(id)
    }

    /// Returns the inner as a usize.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Returns the inner as an f64.
    pub const fn as_f64(self) -> f64 {
        self.0
    }

    /// Returns the inner f64.
    pub const fn inner(self) -> f64 {
        self.0
    }

    /// Returns a dummy, with the f64::MAX inside it.
    pub const fn dummy() -> Self {
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

/// A GmBuffer whose purpose is to pass return data from Rust to GM. Useful in situations
/// where you need to return an [OutputCode], but still have return data that needs to be
/// communicated.
///
/// ## Safety
/// The backing buffer must be *at least* 256 elements in size. This is just because we want to be
/// able to write a large, but not too large amount of things.
///
/// 256 elements, in bytes, is `core::mem::size_of::<u32>() * 256`, or 1 kilobyte.
pub struct Bridge(GmBuffer<u32>);

impl Bridge {
    /// Creates a new [Bridge] based upon a [GmBuffer].
    pub fn new(buf: GmBuffer<u32>) -> Self {
        debug_assert!(
            buf.buffer.len() >= 256,
            "your backing buffer needs to be at least 256 bytes"
        );

        Self(buf)
    }

    /// Creates a new [BridgeWriter] for this [GmBridge].
    pub fn writer(&mut self) -> BridgeWriter<'_> {
        BridgeWriter::new(self)
    }
}

/// A utility for writing into a Bridge. Maintains a cursor, only relevant for its own
/// writes.
pub struct BridgeWriter<'a>(&'a mut Bridge, usize);
impl<'a> BridgeWriter<'a> {
    fn new(bridge: &'a mut Bridge) -> Self {
        Self(bridge, 0)
    }

    /// Writes a u32 into the bridge at the [BridgeWriter]'s current position.
    pub fn write_u32(&mut self, value: u32) {
        self.0 .0[self.1] = value;
        self.1 += 1;
    }

    /// Writes a f32 into the bridge at the [BridgeWriter]'s current position.
    pub fn write_f32(&mut self, value: f32) {
        self.0 .0[self.1] = value.to_bits();
        self.1 += 1;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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

    #[test]
    fn bridge() {
        let buf = vec![0u32; 256];
        let gm_ptr = GmPtr::new(buf.as_ptr() as *const _);

        let mut bridge = unsafe { Bridge::new(GmBuffer::new(GmId::new(0.0), gm_ptr, 256)) };

        let mut writer = bridge.writer();
        writer.write_u32(18);
        writer.write_f32(4.2);

        assert_eq!(buf[0], 18);
        assert_eq!(f32::from_bits(buf[1]), 4.2);

        let mut writer = bridge.writer();
        writer.write_f32(44.3);
        writer.write_f32(22.2);

        assert_eq!(f32::from_bits(buf[0]), 44.3);
        assert_eq!(f32::from_bits(buf[1]), 22.2);
    }
}
