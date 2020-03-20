use std::process::exit;

use replicante_util_failure::format_fail;
use replidev::run;

fn main() {
    match run() {
        Err(error) => {
            let message = format_fail(&error);
            eprintln!("{}", message);
            exit(1);
        }
        Ok(clean) if !clean => exit(1),
        _ => (),
    };
}
