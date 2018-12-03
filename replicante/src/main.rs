extern crate replicante;
extern crate replicante_util_failure;


use std::process::exit;

use replicante::run;
use replicante_util_failure::format_fail;


fn main() {
    if let Err(error) = run() {
        let message = format_fail(&error);
        println!("{}", message);
        exit(1);
    }
}
