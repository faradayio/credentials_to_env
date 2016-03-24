//! A wrapper around the C library's `execv` function.

use errno::{Errno, errno};
use libc;
use std::error::{self, Error};
use std::ffi::{CString, NulError};
use std::iter::{IntoIterator, Iterator};
use std::fmt;
use std::ptr;

/// Represents an error calling `exec`.
#[derive(Debug)]
pub enum ExecError {
    /// One of the strings passed to `execv` contained an internal null byte
    /// and can't be passed correctly to C.
    BadArgument(NulError),
    /// An error was returned by the system.
    Errno(Errno),
}

impl error::Error for ExecError {
    fn description(&self) -> &str {
        match self {
            &ExecError::BadArgument(_) => "bad argument to exec",
            &ExecError::Errno(_) => "couldn't exec process",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ExecError::BadArgument(ref err) => Some(err),
            &ExecError::Errno(_) => None,
        }
    }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ExecError::BadArgument(ref err) =>
                write!(f, "{}: {}", self.description(), err),
            &ExecError::Errno(err) =>
                write!(f, "{}: {}", self.description(), err),
        }
    }
}

impl From<NulError> for ExecError {
    /// Convert a `NulError` into an `ExecError`.
    fn from(err: NulError) -> ExecError {
        ExecError::BadArgument(err)
    }
}

/// Run `program` with `args`, completely replacing the currently running
/// program.
pub fn execvp<'a, S, I>(program: S, args: I) -> Result<(), ExecError>
    where S: AsRef<str>, I: IntoIterator, I::Item: AsRef<str>
{
    // Add null terminations to our strings and our argument array,
    // converting them into a C-compatible format.
    let program_cstring = try!(CString::new(program.as_ref()));
    let arg_cstrings = try!(args.into_iter().map(|arg| {
        CString::new(arg.as_ref())
    }).collect::<Result<Vec<_>, _>>());
    let mut arg_charptrs: Vec<_> = arg_cstrings.iter().map(|arg| {
        arg.as_bytes_with_nul().as_ptr() as *const i8
    }).collect();
    arg_charptrs.push(ptr::null());

    // Use an `unsafe` block so that we can call directly into C.
    let res = unsafe {
        libc::execvp(program_cstring.as_bytes_with_nul().as_ptr() as *const i8,
                     arg_charptrs.as_ptr())
    };

    // Handle our error result.
    if res < 0 {
        Err(ExecError::Errno(errno()))
    } else {
        // Should never happen.
        Ok(())
    }
}
