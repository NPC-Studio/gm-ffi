/// A status code the represents the outcome of a Rust-side function,
/// intended to be sent back to GameMaker.
#[repr(transparent)]
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
impl core::ops::Deref for GmPtr {
    type Target = *const cty::c_char;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
