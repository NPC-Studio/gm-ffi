use std::str::FromStr;

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
pub struct GmPtr(*const cty::c_char);
impl GmPtr {
    /// Creates a new GmPtr based on the given pointer.
    pub fn new(ptr: *const cty::c_char) -> Self {
        Self(ptr)
    }

    /// Returns a copy of the inner value.
    pub fn inner(&self) -> *const cty::c_char {
        self.0
    }

    /// Transforms the inner value into an &str.
    ///
    /// # Saftey
    /// Assumes that the pointer being used is valid as a c_str pointer.
    pub fn to_str(self) -> Result<&'static str, std::str::Utf8Error> {
        unsafe { cstr_core::CStr::from_ptr(self.0).to_str() }
    }
}
impl FromStr for GmPtr {
    type Err = cstr_core::NulError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GmPtr(cstr_core::CString::new(s)?.as_ptr()))
    }
}
impl core::ops::Deref for GmPtr {
    type Target = *const cty::c_char;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
