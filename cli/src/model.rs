use std::{env, io, path::PathBuf};

use clap::ValueEnum;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[value(rename_all = "lower")]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    OpenCode,
    Claude,
    Codex,
    Pi,
    Gemini,
    Amp,
    Goose,
    Qwen,
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OpenCode => "opencode",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Pi => "pi",
            Self::Gemini => "gemini",
            Self::Amp => "amp",
            Self::Goose => "goose",
            Self::Qwen => "qwen",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Global,
    ProjectRoot,
    Directory,
    Local,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Global => "GLOBAL",
            Self::ProjectRoot => "PROJECT ROOT",
            Self::Directory => "DIRECTORY",
            Self::Local => "LOCAL",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum State {
    Selected,
    Shadowed { by: PathBuf },
    Excluded,
}

#[derive(Clone, Debug, Serialize)]
pub struct Candidate {
    pub path: PathBuf,
    pub scope: Scope,
    pub reason: String,
    pub state: State,
}

#[derive(Clone, Debug, Serialize)]
pub struct Resolution {
    pub agent: Agent,
    pub dir: PathBuf,
    pub boundary: String,
    pub candidates: Vec<Candidate>,
    pub checked: Vec<String>,
    pub notes: Vec<String>,
}

impl Resolution {
    pub fn selected(&self) -> impl Iterator<Item = &Candidate> {
        self.candidates
            .iter()
            .filter(|c| c.state == State::Selected)
    }

    pub fn excluded(&self) -> impl Iterator<Item = &Candidate> {
        self.candidates
            .iter()
            .filter(|c| !matches!(c.state, State::Selected))
    }
}

#[derive(Clone, Debug)]
pub struct CodexConfig {
    pub home: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct PiConfig {
    pub directory: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct OpenCodeConfig {
    pub disable_claude: bool,
    pub disable_claude_prompt: bool,
    pub disable_project_config: bool,
}

#[derive(Clone, Debug)]
pub struct ResolverConfig {
    pub home: PathBuf,
    pub fs_root: Option<PathBuf>,
    pub codex: CodexConfig,
    pub pi: PiConfig,
    pub opencode: OpenCodeConfig,
}

impl ResolverConfig {
    pub fn from_env() -> io::Result<Self> {
        fn flag(name: &str) -> bool {
            matches!(env::var(name).as_deref(), Ok("true" | "1"))
        }

        let home = env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine home directory",
            )
        })?;
        let pi_directory = env::var_os("PI_CODING_AGENT_DIR")
            .map(PathBuf::from)
            .map(|path| match path.strip_prefix("~") {
                Ok(relative) => home.join(relative),
                Err(_) => path,
            });

        Ok(Self {
            home,
            fs_root: None,
            codex: CodexConfig {
                home: env::var_os("CODEX_HOME").map(PathBuf::from),
            },
            pi: PiConfig {
                directory: pi_directory,
            },
            opencode: OpenCodeConfig {
                disable_claude: flag("OPENCODE_DISABLE_CLAUDE_CODE"),
                disable_claude_prompt: flag("OPENCODE_DISABLE_CLAUDE_CODE_PROMPT"),
                disable_project_config: matches!(env::var("OPENCODE_DISABLE_PROJECT_CONFIG"), Ok(value) if value == "true" || value == "1"),
            },
        })
    }
}
