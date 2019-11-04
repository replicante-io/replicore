use std::process::exit;

use replicante_util_failure::format_fail;
use repliadm::run;

fn main() {
    if let Err(error) = run() {
        let message = format_fail(&error);
        println!("{}", message);
        exit(1);
    }
}
