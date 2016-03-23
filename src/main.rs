extern crate credentials;

use credentials::Secretfile;
use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
use std::process;

/// A nice, generic error type which can hold any error returned by any
/// library we use, and to which the `try!` macro will automatically
/// convert error types.  This is a common Rust trick.
pub type Error = Box<error::Error+Send+Sync>;

/// This function does all the real work, and returns any errors to main,
/// which handles them all in one place.
fn helper() -> Result<(), Error> {
    let secretfile = try!(Secretfile::default());

    // Copy the environment variables listed in Secretfile to our local
    // environment.
    for var in secretfile.vars() {
        env::set_var(&var, &try!(credentials::var(&var)));
    }

    // Copy the files listed in Secretfile to our local file system.
    for path in secretfile.files() {
        let data = try!(credentials::file(path));
        let mut f = try!(fs::File::create(path));
        try!(f.write_all(data.as_bytes()));
    }

    Ok(())
}

/// An error-handling wrapper around `helper`.
fn main() {
    if let Err(err) = helper() {
        writeln!(&mut io::stderr(), "Error: {}", err).unwrap();
        process::exit(1);
    }
}
