use crate::conf::Project;

#[derive(thiserror::Error, Debug)]
#[error("The replidev {command} command does not support the {project} project")]
pub struct InvalidProject {
    command: &'static str,
    project: Project,
}

impl InvalidProject {
    pub fn new(project: Project, command: &'static str) -> InvalidProject {
        InvalidProject { command, project }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("The release check process raised some issues")]
pub struct ReleaseCheck {
    /// List of errors raised by the release check process.
    pub errors: Vec<anyhow::Error>,
}

impl ReleaseCheck {
    /// Check a result for `ReleaseCheck` and expand the collections of errors.
    ///
    /// If the result failed with a `ReleaseCheck` error then the errors in it are
    /// added to the errors in this instance and `Ok(())` is return.
    ///
    /// Any other error is returned unchanged.
    pub fn check(&mut self, result: anyhow::Result<()>) -> anyhow::Result<()> {
        let error = match result {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        match error.downcast::<ReleaseCheck>() {
            Err(error) => Err(error),
            Ok(issues) => {
                self.errors.extend(issues.errors);
                Ok(())
            }
        }
    }

    /// Create a `ReleaseCheck` containing the given error.
    pub fn failed<E>(error: E) -> anyhow::Result<()>
    where
        E: Into<anyhow::Error>,
    {
        let errors = vec![error.into()];
        let error = anyhow::anyhow!(ReleaseCheck { errors });
        Err(error)
    }

    /// Convert this collection of errors into a result.
    ///
    /// If no errors were collected returns `Ok(())` otherwise returns itself as an error.
    pub fn into_result(self) -> anyhow::Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(self))
        }
    }

    pub fn new() -> ReleaseCheck {
        ReleaseCheck { errors: Vec::new() }
    }
}

impl From<anyhow::Error> for ReleaseCheck {
    fn from(error: anyhow::Error) -> ReleaseCheck {
        let errors = vec![error];
        ReleaseCheck { errors }
    }
}
