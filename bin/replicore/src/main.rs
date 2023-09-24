use replicore::run;

fn main() {
    let result = run();

    // Default error handling prints the error in detailed format.
    if let Err(error) = result {
        eprintln!("Replicante Core process failed: {:?}", error);
        std::process::exit(1);
    }
}
