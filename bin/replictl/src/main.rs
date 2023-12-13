use replictl::errors::ContextNotFound;
use replictl::errors::InvalidScope;
use replictl::run;

#[tokio::main]
async fn main() {
    // Run the CLI.
    let result = run().await;

    // Exit if the CLI returned an exit code for us to use.
    let error = match result {
        Err(error) => error,
        Ok(0) => return,
        Ok(code) => std::process::exit(code),
    };

    // Provide best possible help for known errors or fallback to default defaulted format.
    format_known_errors(&error);
    eprintln!("Replicante Core process failed: {:?}", error);
    std::process::exit(1);
}

/// Format well known errors, such as API responses, to provide a better user experience.
///
/// ## Early exit
///
/// The function terminates the process early if it finds a well known error.
/// This is done to allow different return codes based on the error and to keep the
/// [`main`] control flow more linear.
fn format_known_errors(error: &anyhow::Error) {
    if let Some(error) = error.downcast_ref::<ContextNotFound>() {
        eprintln!("{}", error);
        eprintln!(
            "You will need to create the context with 'replictl context configure --context {}'",
            error.name(),
        );
        std::process::exit(1);
    }
    if let Some(error) = error.downcast_ref::<InvalidScope>() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

//    if let Some(replictl::ApiNotFound) = error.downcast_ref() {
//        eprintln!("{:?}", error);
//        eprintln!("Below is a PARTIAL list of likely causes for this error:");
//        eprintln!("  * The request was operating on a resource that does not exist (example: a missing cluster)");
//        eprintln!("  * The current session may have expired or be otherwise not valid (please login again)");
//        eprintln!("  * The version of Replicante Core does not support the request (try to keep replictl to the same version as your running clusters)");
//        std::process::exit(1);
//    }
//    if let Some(errors) = error.downcast_ref::<replictl::InvalidApply>() {
//        eprintln!("Unable to apply the proveded object:");
//        for error in errors.iter() {
//            eprintln!(
//                "  -> {} [attribute={}; code={}]",
//                error.message, error.attribute, error.code
//            );
//        }
//        std::process::exit(1);
//    }
