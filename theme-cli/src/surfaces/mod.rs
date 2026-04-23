pub mod apps;
pub mod chrome;
pub mod gnome_terminal;
pub mod nvim;
pub mod system;
pub mod tmux;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

impl Mode {
    pub fn flip(self) -> Self {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }

    /// gsettings color-scheme value.
    pub fn color_scheme(self) -> &'static str {
        match self {
            Mode::Light => "prefer-light",
            Mode::Dark => "prefer-dark",
        }
    }
}

pub struct SurfaceReport {
    pub name: &'static str,
    pub outcome: anyhow::Result<String>,
}

impl SurfaceReport {
    pub fn ok(name: &'static str, msg: impl Into<String>) -> Self {
        Self { name, outcome: Ok(msg.into()) }
    }

    pub fn err(name: &'static str, err: anyhow::Error) -> Self {
        Self { name, outcome: Err(err) }
    }
}
