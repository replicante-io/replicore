use replictl::run;

fn main() {
    let result = run();
    let error = match result {
        Err(error) => error,
        Ok(0) => return,
        Ok(num) => std::process::exit(num),
    };

    // Provide better error messages for cases where we can provide suggestions to the user.
    if let Some(replictl::ApiNotFound) = error.downcast_ref() {
        eprintln!("{:?}", error);
        eprintln!("Below is a PARTIAL list of likely causes for this error:");
        eprintln!("  * The request was operating on a resource that does not exist (example: a missing cluster)");
        eprintln!("  * The current session may have expired or be otherwise not valid (please login again)");
        eprintln!("  * The version of Replicante Core does not support the request (try to keep replictl to the same version as your running clusters)");
        std::process::exit(1);
    }
    if let Some(error) = error.downcast_ref::<replictl::ContextNotFound>() {
        eprintln!("{}", error);
        eprintln!(
            "Try logging in with 'replictl context login --context {}'",
            error.name(),
        );
        std::process::exit(1);
    }
    if let Some(errors) = error.downcast_ref::<replictl::InvalidApply>() {
        eprintln!("Unable to apply the proveded object:");
        for error in errors.iter() {
            eprintln!(
                "  -> {} [attribute={}; code={}]",
                error.message, error.attribute, error.code
            );
        }
        std::process::exit(1);
    }
    if let Some(error) = error.downcast_ref::<replictl::ScopeError>() {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    // Print the error in detailed format for all other cases.
    eprintln!("{:?}", error);
    std::process::exit(1);
}
