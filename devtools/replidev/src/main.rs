use replicante_util_failure::format_fail;

fn main() {
    let result = replidev::run();
    let error = match result {
        Err(error) => error,
        Ok(0) => return,
        Ok(num) => std::process::exit(num),
    };

    // Provide better error messages for cases where we can provide suggestions.
    if let Some(error) = error.downcast_ref::<replidev::error::ReleaseCheck>() {
        match error.errors.len() {
            0 => eprintln!("{}", error),
            1 => eprintln!("{}", error.errors[0]),
            _ => {
                eprintln!("{}:", error);
                for error in &error.errors {
                    eprintln!("  * {}", error);
                }
            }
        }
        std::process::exit(1);
    }

    // Fallback to failure errors while we migrate to anyhow.
    if let Some(error) = error.downcast_ref::<replidev::Error>() {
        let message = format_fail(error);
        eprintln!("{}", message);
        std::process::exit(1);
    }

    // Print the error in detailed format for all other cases.
    eprintln!("{:?}", error);
    std::process::exit(1);
}
