extern crate replicante_util_failure;
extern crate replictl;

use std::env;
use std::process::exit;

use replicante_util_failure::format_fail;
use replictl::run;

fn main() {
    // Enable backtraces if the user did not set them.
    let have_rust = env::var("RUST_BACKTRACE").is_ok();
    let have_failure = env::var("RUST_FAILURE_BACKTRACE").is_ok();
    if !have_rust && !have_failure {
        env::set_var("RUST_FAILURE_BACKTRACE", "1");
    }

    // Can now run replictl.
    if let Err(error) = run() {
        let message = format_fail(&error);
        println!("{}", message);
        exit(1);
    }
}
