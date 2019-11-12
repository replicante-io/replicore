use std::process::exit;

use replicante_util_failure::format_fail;
use replictl::run;
use replictl::ErrorKind;
use replictl::CLI_NAME;

fn main() {
    if let Err(error) = run() {
        if let ErrorKind::SessionNotFound(ref session) = error.kind() {
            println!("No SSO session named '{}' was found", session);
            println!("Try logging in with `{} sso login`", CLI_NAME);
        } else {
            let message = format_fail(&error);
            println!("{}", message);
        }
        exit(1);
    }
}
