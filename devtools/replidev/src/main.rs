use replicante_util_failure::format_fail;
use replidev::run;

fn main() {
    match run() {
        Err(error) => {
            let message = format_fail(&error);
            eprintln!("{}", message);
            std::process::exit(1);
        }
        Ok(0) => (),
        Ok(num) => std::process::exit(num),
    };
}
