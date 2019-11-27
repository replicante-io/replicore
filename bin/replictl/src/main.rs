use std::process::exit;

use replicante_util_failure::format_fail;
use replictl::run;
use replictl::ErrorKind;
use replictl::CLI_NAME;

fn main() {
    if let Err(error) = run() {
        match error.kind() {
            ErrorKind::ApplyValidation(errors) => {
                eprintln!("Unable to apply the proveded object:");
                for error in errors.iter() {
                    eprintln!(
                        "  -> {} [attribute={}; code={}]",
                        error.message, error.attribute, error.code
                    );
                }
            }
            ErrorKind::RepliClientNotFound => {
                let message = format_fail(&error);
                eprintln!("{}", message);
                eprintln!("Below is a PARTIAL list of likely causes for this error:");
                eprintln!("  * The request was operating on a resource that does not exist (example: a missing cluster)");
                eprintln!("  * The current SSO session may have expired or be otherwise not valid (logout then login again)");
                eprintln!("  * The version of Replicante Core does not support the request (try to keep replictl to the same version as your running clusters)");
            }
            ErrorKind::SessionNotFound(ref session) => {
                eprintln!("No SSO session named '{}' was found", session);
                eprintln!("Try logging in with `{} sso login`", CLI_NAME);
            }
            _ => {
                let message = format_fail(&error);
                eprintln!("{}", message);
            }
        }
        exit(1);
    }
}
