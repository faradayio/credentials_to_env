//! A wrapper around `chmod`.  I think there may be a version of this
//! planned for a future version of Rust's `std`.
//!
//! This is not an especially outstanding C wrapper, but I'm not in the
//! mood to try to solve this problem in its full generality, so a little
//! local wrapper like this is fine.

use errno::{Errno, errno};
use libc;
use std::error::{self, Error};
use std::ffi::{CString, NulError};
use std::fmt;

/// Represents an error calling `exec`.
#[derive(Debug)]
pub enum ChmodError {
    /// One of the strings passed to `execv` contained an internal null byte
    /// and can't be passed correctly to C.
    BadArgument(NulError),
    /// An error was returned by the system.
    Errno(Errno),
}

impl error::Error for ChmodError {
    fn description(&self) -> &str {
        match self {
            &ChmodError::BadArgument(_) => "bad argument to chmod",
            &ChmodError::Errno(_) => "couldn't chmod file",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ChmodError::BadArgument(ref err) => Some(err),
            &ChmodError::Errno(_) => None,
        }
    }
}

impl fmt::Display for ChmodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ChmodError::BadArgument(ref err) =>
                write!(f, "{}: {}", self.description(), err),
            &ChmodError::Errno(err) =>
                write!(f, "{}: {}", self.description(), err),
        }
    }
}

impl From<NulError> for ChmodError {
    /// Convert a `NulError` into an `ChmodError`.
    fn from(err: NulError) -> ChmodError {
        ChmodError::BadArgument(err)
    }
}

/// Change the permissions of a file.
///
/// NOTE: This API is not very good.  It should use Path instead of String,
/// for example.
pub fn chmod<S: Into<String>>(path: S, permissions: libc::mode_t) ->
    Result<(), ChmodError>
{
    // Add null terminations to our strings and our argument array,
    // converting them into a C-compatible format.
    let path_cstring = try!(CString::new(path.into()));
    
    // Use an `unsafe` block so that we can call directly into C.
    let res = unsafe {
        libc::chmod(path_cstring.as_bytes_with_nul().as_ptr() as *const i8,
                    permissions)
    };

    // Handle our error result.
    if res < 0 {
        Err(ChmodError::Errno(errno()))
    } else {
        // Everything worked fine.
        Ok(())
    }
}
