#[macro_use]
extern crate common_failures;
extern crate credentials;
extern crate exec;
extern crate failure;

use common_failures::prelude::*;
use credentials::{Client, Options, Secretfile};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::os::unix::fs::PermissionsExt;

/// Our command-line arguments.
struct Args {
    allow_override: bool,
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
  credentials-to-env [--no-env-override] [-f <secretfile>] <app> [<args>...]

Processes a Secretfile, loading secrets into the environment or writing
them to files as requested.  Once this is done, it execs <app> with <args>.

Options:
  --no-env-override  Do not allow environement variables to override
                     Secretfile contents.
  -f <secretfile>    Use the specified Secretfile.  Defaults to
                     `Secretfile` in the current directory.

For more information, see https://github.com/faradayio/credentials_to_env
");
        process::exit(exit_code)
    }

    /// Parse our command-line arguments, and exit on errors, `--help` or
    /// `--version`.  We use our own argument parser because this kind of
    /// compound "forwarding to a second app" command-line tends to be more
    /// trouble than it's worth even with off-the-shelf tools.
    fn parse<I>(arguments: I) -> Args
        where I: IntoIterator, I::Item: AsRef<str>
    {
        let mut args: Vec<String> =
            arguments.into_iter().map(|s| s.as_ref().to_owned()).collect();
        let mut secretfile = None;
        let mut allow_override = true;
        while !args.is_empty() {
            match args[0].as_ref() {
                "--help" => Args::usage(0),
                "--version" => {
                    // `env!` fetches compile-time env variables.
                    // CARGO_PKG_VERSION is set by Cargo during the build.
                    println!("credentials-to-env {}", env!("CARGO_PKG_VERSION"));
                    process::exit(0);
                }
                "-f" if args.len() >= 2 => {
                    secretfile = Some(Path::new(&args[1]).to_owned());
                    args.remove(0);
                    args.remove(0);
                }
                "--no-env-override" => {
                    allow_override = false;
                    args.remove(0);
                }
                "--" => {
                    args.remove(0);
                    break;
                }
                _ => {
                    if args[0].chars().next() == Some('-') {
                        // Unknown '-' argument, so bail.
                        Args::usage(1);
                    } else {
                        // We've found a non-option argument, so stop
                        // looking for options.
                        break;
                    }
                }
            }
        }

        // Make sure we have at least one more argument, and that it
        // doesn't start with "-".
        if args.is_empty() || args[0].chars().next() == Some('-') {
            Args::usage(1)
        }
        let program = args.remove(0);

        Args {
            allow_override: allow_override,
            secretfile: secretfile,
            program: program,
            args: args,
        }
    }
}

#[test]
fn test_args_parse() {
    let args = Args::parse(&["foo"]);
    assert_eq!(true, args.allow_override);
    assert_eq!(None, args.secretfile);
    assert_eq!("foo", args.program);
    assert_eq!(vec!() as Vec<String>, args.args);

    let args = Args::parse(&["--no-env-override", "-f", "/app/Secretfile",
                             "--", "foo", "--bar"]);
    assert_eq!(false, args.allow_override);
    assert_eq!(Some(Path::new("/app/Secretfile").to_owned()), args.secretfile);
    assert_eq!("foo", args.program);
    assert_eq!(vec!("--bar"), args.args);
}

/// This function does all the real work, and returns any errors to main,
/// which handles them all in one place.
fn run() -> Result<()> {
    // Fetch our arguments.
    let args = Args::parse(env::args().skip(1));

    // Get our Secretfile and construct a client.
    let secretfile = try!(match &args.secretfile {
        &Some(ref path) => Secretfile::from_path(path),
        &None => Secretfile::default(),
    });
    let options = Options::default()
        .secretfile(secretfile.clone())
        .allow_override(args.allow_override);
    let mut client = try!(Client::new(options));

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

    // Execute the command we were passed. This returns an error if it returns
    // at all, which we wrap in `Err` to make a `Result` (for easier
    // processing).
    Err(exec::Command::new(&args.program).args(&args.args).exec())
        .context("could not execute specified program")
        .map_err(|e| e.into())
}

/// An error-handling wrapper around `run`.
quick_main!(run);
