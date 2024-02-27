use std::process::exit;

use repliadm::run;
use replicante_util_failure::format_fail;

fn main() {
    if let Err(error) = run() {
        let message = format_fail(&error);
        eprintln!("{}", message);
        exit(1);
    }
}
