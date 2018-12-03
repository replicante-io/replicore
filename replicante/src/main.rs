extern crate replicante;
extern crate replicante_util_failure;


use std::env;
use std::process::exit;

use replicante::run;
use replicante_util_failure::format_fail;


fn main() {
    // Enable backtraces if the user did not set them.
    let have_rust = env::var("RUST_BACKTRACE").is_ok();
    let have_failure = env::var("RUST_FAILURE_BACKTRACE").is_ok();
    if !have_rust && !have_failure {
        env::set_var("RUST_FAILURE_BACKTRACE", "1");
    }

    // Can now run replicante.
    if let Err(error) = run() {
        let message = format_fail(&error);
        println!("{}", message);
        exit(1);
    }
}
