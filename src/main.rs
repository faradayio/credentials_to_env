extern crate credentials;
extern crate errno;
extern crate libc;

use credentials::Secretfile;
use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

mod chmod;
mod exec;

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
    for path_str in secretfile.files() {
        let path = Path::new(path_str);

        // Don't overwrite a file which already exists.
        if !path.exists() {
            // Make sure the directory exists.
            if let Some(parent) = path.parent() {
                try!(fs::create_dir_all(parent));
            }

            // Write the data to a file that's only readable by us.
            let data = try!(credentials::file(path));
            let mut f = try!(fs::File::create(path));
            try!(chmod::chmod(path_str.clone(), 0o400));
            try!(f.write_all(data.as_bytes()));
        }
    }

    // If we were supplied with command-line arguments, treat them as a
    // command to exec.  This will replace the currently running binary.
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let program = args[1].clone();
        try!(exec::execvp(program, &args[1..]));
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
