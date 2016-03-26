extern crate credentials;
extern crate errno;
extern crate exec;
extern crate libc;

use credentials::{Client, Secretfile};
use std::env;
use std::error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::os::unix::fs::PermissionsExt;

/// A nice, generic error type which can hold any error returned by any
/// library we use, and to which the `try!` macro will automatically
/// convert error types.  This is a common Rust trick.
pub type Error = Box<error::Error+Send+Sync>;

/// Our command-line arguments.
struct Args {
    secretfile: Option<PathBuf>,
    program: String,
    args: Vec<String>,
}

impl Args {
    /// Display a usage message, and exit with the specified code.
    fn usage(exit_code: i32) -> ! {
        println!("\
Usage:
  credentials-to-env --version
  credentials-to-env --help
  credentials-to-env [-f <secretfile>] <app> [<args>...]

Processes either the specified Secretfile, or the Secretfile in the current
directory, loading secrets into the environment or writing them to files as
requested.  Once this is done, it execs <app> with <args>.

For more information, see https://github.com/faradayio/credentials_to_env
");
        process::exit(exit_code)
    }

    /// Parse our command-line arguments, and exit on errors, `--help` or
    /// `--version`.  We use our own argument parser because this kind of
    /// compound "forwarding to a second app" command-line tends to be more
    /// trouble than it's worth even with off-the-shelf tools.
    fn parse() -> Result<Args, Error> {
        let mut args: Vec<String> = env::args().skip(1).collect();
        let mut secretfile = None;
        match args.get(0).map(|s| &s[..]) {
            Some("--help") => Args::usage(0),
            Some("--version") => {
                // `env!` fetches compile-time env variables.
                // CARGO_PKG_VERSION is set by Cargo during the build.
                println!("credentials-to-env {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            Some("-f") if args.len() >= 2 => {
                secretfile = Some(Path::new(&args[1]).to_owned());
                args.remove(0);
                args.remove(0);
            }
            Some(_) => {},
            None => {}
        }

        // Make sure we have at least one more argument, and that it
        // doesn't start with "-".
        if args.is_empty() || args[0].chars().next() == Some('-') {
            Args::usage(1)
        }
        let program = args.remove(0);

        Ok(Args {
            secretfile: secretfile,
            program: program,
            args: args,
        })
    }
}

/// This function does all the real work, and returns any errors to main,
/// which handles them all in one place.
fn helper() -> Result<(), Error> {
    // Fetch our arguments.
    let args = try!(Args::parse());

    // Get our Secretfile and construct a client.
    let secretfile = try!(match &args.secretfile {
        &Some(ref path) => Secretfile::from_path(path),
        &None => Secretfile::default(),
    });
    let mut client = try!(Client::with_secretfile(secretfile.clone()));

    // Copy the environment variables listed in Secretfile to our local
    // environment.
    for var in secretfile.vars() {
        env::set_var(&var, &try!(client.var(&var)));
    }

    // Copy the files listed in Secretfile to our local file system.
    for path_str in secretfile.files() {
        let path = Path::new(&path_str);

        // Don't overwrite a file which already exists.
        if !path.exists() {
            // Make sure the directory exists.
            if let Some(parent) = path.parent() {
                try!(fs::create_dir_all(parent));
            }

            // Write the data to a file that's only readable by us.
            let data = try!(client.file(path));
            let mut f = try!(fs::File::create(path));
            try!(fs::set_permissions(&path_str,
                                     PermissionsExt::from_mode(0o400)));
            try!(f.write_all(data.as_bytes()));
        }
    }

    // Execute the command we were passed.
    let err = exec::Command::new(&args.program).args(&args.args).exec();
    Err(From::from(err))
}

/// An error-handling wrapper around `helper`.
fn main() {
    if let Err(err) = helper() {
        writeln!(&mut io::stderr(), "Error: {}", err).unwrap();
        process::exit(1);
    }
}
