use serde::Deserialize;

/// Supported replidev projects.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum Project {
    /// Replicante Agents Repository
    #[serde(rename = "agents")]
    Agents,

    /// Replicante Common crates for both core and agents
    #[serde(rename = "common")]
    Common,

    /// Replicante Core
    #[serde(rename = "core")]
    Core,

    /// Replicante Playgrounds Project
    #[serde(rename = "playground")]
    Playground,

    /// Replicante WebUI Project
    #[serde(rename = "webui")]
    WebUI,
}

impl Project {
    /// Check if a project is allowed to execute the `deps` family of commands.
    pub fn allow_deps(&self) -> bool {
        *self == Self::Core
    }

    /// Check if a project is allowed to execute the `gen-certs` family of commands.
    pub fn allow_gen_certs(&self) -> bool {
        matches!(self, Self::Agents | Self::Core | Self::Playground)
    }

    /// Check if a project is allowed to execute the `play` family of commands.
    pub fn allow_play(&self) -> bool {
        *self == Self::Playground
    }

    /// Check if a project is allowed to execute the `release` family of commands.
    pub fn allow_release(&self) -> bool {
        matches!(self, Self::Agents | Self::Common | Self::Core | Self::WebUI)
    }

    /// Search for rust crates during the release process of this project.
    pub fn search_for_crates(&self) -> bool {
        matches!(self, Self::Agents | Self::Common | Self::Core)
    }

    /// Search for npm packages during the release process of this project.
    pub fn search_for_npm(&self) -> bool {
        *self == Self::WebUI
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Agents => write!(fmt, "agents"),
            Self::Common => write!(fmt, "common"),
            Self::Core => write!(fmt, "core"),
            Self::Playground => write!(fmt, "playground"),
            Self::WebUI => write!(fmt, "webui"),
        }
    }
}
