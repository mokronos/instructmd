use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
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
        write!(
            f,
            "{}",
            match self {
                Self::OpenCode => "opencode",
                Self::Claude => "claude",
                Self::Codex => "codex",
                Self::Pi => "pi",
                Self::Gemini => "gemini",
                Self::Amp => "amp",
                Self::Goose => "goose",
                Self::Qwen => "qwen",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    Selected,
    Shadowed { by: PathBuf },
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub path: PathBuf,
    pub scope: Scope,
    pub reason: String,
    pub state: State,
}
#[derive(Clone, Debug)]
pub struct Resolution {
    pub agent: Agent,
    pub dir: PathBuf,
    pub boundary: String,
    pub candidates: Vec<Candidate>,
    pub checked: Vec<String>,
}
impl Resolution {
    pub fn selected(&self) -> impl Iterator<Item = &Candidate> {
        self.candidates
            .iter()
            .filter(|c| c.state == State::Selected)
    }
}

#[derive(Clone, Debug)]
pub struct ResolverConfig {
    pub home: PathBuf,
    pub codex_home: Option<PathBuf>,
    pub pi_dir: Option<PathBuf>,
    pub fs_root: Option<PathBuf>,
    pub opencode_disable_claude: bool,
    pub opencode_disable_claude_prompt: bool,
}
impl ResolverConfig {
    pub fn from_env() -> io::Result<Self> {
        fn flag(name: &str) -> bool {
            env::var_os(name).is_some_and(|v| !v.is_empty() && v != "0")
        }
        Ok(Self {
            home: env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not determine home directory",
                )
            })?,
            codex_home: env::var_os("CODEX_HOME").map(PathBuf::from),
            pi_dir: env::var_os("PI_CODING_AGENT_DIR").map(PathBuf::from),
            fs_root: None,
            opencode_disable_claude: flag("OPENCODE_DISABLE_CLAUDE_CODE"),
            opencode_disable_claude_prompt: flag("OPENCODE_DISABLE_CLAUDE_CODE_PROMPT"),
        })
    }
}

pub fn resolve(agent: Agent, dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    match agent {
        Agent::OpenCode => opencode(dir, cfg),
        Agent::Claude => claude(dir, cfg),
        Agent::Codex => codex(dir, cfg),
        Agent::Pi => pi(dir, cfg),
        Agent::Gemini => gemini(dir, cfg),
        Agent::Amp => amp(dir, cfg),
        Agent::Goose => goose(dir, cfg),
        Agent::Qwen => qwen(dir, cfg),
    }
}

fn base(agent: Agent, dir: PathBuf, boundary: String, checked: &[&str]) -> Resolution {
    Resolution {
        agent,
        dir,
        boundary,
        candidates: vec![],
        checked: checked.iter().map(|s| (*s).into()).collect(),
    }
}
fn exists(p: &Path) -> bool {
    p.is_file()
}
fn scope_for(p: &Path, root: &Path, dir: &Path) -> Scope {
    if p == dir {
        Scope::Directory
    } else if p == root {
        Scope::ProjectRoot
    } else {
        Scope::Directory
    }
}
fn chain(root: &Path, dir: &Path) -> Vec<PathBuf> {
    let mut v = vec![];
    let mut p = dir.to_path_buf();
    loop {
        v.push(p.clone());
        if p == root {
            break;
        }
        if !p.pop() {
            break;
        }
    }
    v.reverse();
    v
}
fn git_root(dir: &Path, floor: &Path) -> Option<PathBuf> {
    let mut p = dir.to_path_buf();
    loop {
        if p.join(".git").exists() {
            return Some(p);
        }
        if p == floor || !p.pop() {
            return None;
        }
    }
}
fn fs_root(dir: &Path, cfg: &ResolverConfig) -> PathBuf {
    cfg.fs_root
        .clone()
        .unwrap_or_else(|| dir.ancestors().last().unwrap_or(dir).to_path_buf())
}
fn add_first(
    r: &mut Resolution,
    directory: &Path,
    context: (&Path, &Path),
    names: &[&str],
    reason: &str,
    local: bool,
    nonempty: bool,
) {
    let found: Vec<PathBuf> = names
        .iter()
        .map(|n| directory.join(n))
        .filter(|p| exists(p))
        .collect();
    let winner = found
        .iter()
        .find(|p| !nonempty || fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false))
        .cloned();
    for p in found {
        if Some(&p) == winner.as_ref() {
            r.candidates.push(Candidate {
                path: p,
                scope: if local {
                    Scope::Local
                } else {
                    scope_for(directory, context.0, context.1)
                },
                reason: reason.into(),
                state: State::Selected,
            });
        } else {
            let why = winner.clone().unwrap_or_else(|| p.clone());
            let empty = fs::metadata(&p).map(|m| m.len() == 0).unwrap_or(true);
            r.candidates.push(Candidate {
                path: p,
                scope: if local {
                    Scope::Local
                } else {
                    scope_for(directory, context.0, context.1)
                },
                reason: if nonempty && empty {
                    "existing but empty; skipped".into()
                } else {
                    "lost same-directory candidate selection".into()
                },
                state: State::Shadowed { by: why },
            });
        }
    }
}
fn add_all(
    r: &mut Resolution,
    directory: &Path,
    root: &Path,
    target: &Path,
    names: &[(&str, bool)],
    reason: &str,
) {
    for (name, local) in names {
        let p = directory.join(name);
        if exists(&p) {
            r.candidates.push(Candidate {
                path: p,
                scope: if *local {
                    Scope::Local
                } else {
                    scope_for(directory, root, target)
                },
                reason: reason.into(),
                state: State::Selected,
            });
        }
    }
}
fn add_global_first(r: &mut Resolution, dir: &Path, names: &[&str], reason: &str, nonempty: bool) {
    let start = r.candidates.len();
    add_first(r, dir, (dir, dir), names, reason, false, nonempty);
    for c in &mut r.candidates[start..] {
        c.scope = Scope::Global;
    }
}
fn add_global_first_paths(r: &mut Resolution, paths: &[PathBuf], reason: &str, nonempty: bool) {
    let found: Vec<PathBuf> = paths.iter().filter(|p| exists(p)).cloned().collect();
    let winner = found
        .iter()
        .find(|p| !nonempty || fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false))
        .cloned();
    for path in found {
        if Some(&path) == winner.as_ref() {
            r.candidates.push(Candidate {
                path,
                scope: Scope::Global,
                reason: reason.into(),
                state: State::Selected,
            });
        } else {
            let empty = fs::metadata(&path).map(|m| m.len() == 0).unwrap_or(true);
            r.candidates.push(Candidate {
                path: path.clone(),
                scope: Scope::Global,
                reason: if nonempty && empty {
                    "existing but empty; skipped".into()
                } else {
                    "lost global candidate selection".into()
                },
                state: State::Shadowed {
                    by: winner.clone().unwrap_or(path),
                },
            });
        }
    }
}
fn add_global(r: &mut Resolution, p: PathBuf, reason: &str) {
    if exists(&p) {
        r.candidates.push(Candidate {
            path: p,
            scope: Scope::Global,
            reason: reason.into(),
            state: State::Selected,
        });
    }
}

fn opencode(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let disable_all = cfg.opencode_disable_claude;
    let disable_prompt = disable_all || cfg.opencode_disable_claude_prompt;
    let root = git_root(&dir, &fs_root(&dir, cfg));
    let project = root.clone().unwrap_or_else(|| dir.clone());
    let mut r = base(
        Agent::OpenCode,
        dir.clone(),
        root.map(|p| format!("git root: {}", p.display()))
            .unwrap_or_else(|| "no git root; project directory only".into()),
        &[
            "~/.config/opencode/AGENTS.md",
            "~/.claude/CLAUDE.md (unless disabled)",
            "AGENTS.md | CLAUDE.md (unless disabled) | CONTEXT.md",
        ],
    );
    if disable_prompt {
        add_global(
            &mut r,
            cfg.home.join(".config/opencode/AGENTS.md"),
            "global instruction location; ~/.claude/CLAUDE.md fallback disabled by environment",
        );
    } else {
        add_global_first_paths(
            &mut r,
            &[
                cfg.home.join(".config/opencode/AGENTS.md"),
                cfg.home.join(".claude/CLAUDE.md"),
            ],
            "global instruction location",
            false,
        );
    }
    let names: &[&str] = if disable_all {
        &["AGENTS.md", "CONTEXT.md"]
    } else {
        &["AGENTS.md", "CLAUDE.md", "CONTEXT.md"]
    };
    for d in chain(&project, &dir) {
        let ctx_wins = exists(&d.join("CONTEXT.md"))
            && !exists(&d.join("AGENTS.md"))
            && (disable_all || !exists(&d.join("CLAUDE.md")));
        add_first(
            &mut r,
            &d,
            (&project, &dir),
            names,
            if ctx_wins {
                "ancestor walk from git root to --dir; first candidate match in this directory (deprecated)"
            } else {
                "ancestor walk from git root to --dir; first candidate match in this directory"
            },
            false,
            false,
        );
    }
    r
}
fn claude(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = fs_root(&dir, cfg);
    let mut r = base(
        Agent::Claude,
        dir.clone(),
        format!("filesystem root: {}", root.display()),
        &[
            "~/.claude/CLAUDE.md",
            "CLAUDE.md + CLAUDE.local.md + .claude/CLAUDE.md",
        ],
    );
    add_global(
        &mut r,
        cfg.home.join(".claude/CLAUDE.md"),
        "global instruction location",
    );
    for d in chain(&root, &dir) {
        add_all(
            &mut r,
            &d,
            &root,
            &dir,
            &[("CLAUDE.md", false), ("CLAUDE.local.md", true)],
            "filesystem-root ancestor walk; files compose",
        );
        let p = d.join(".claude/CLAUDE.md");
        if exists(&p) && !r.candidates.iter().any(|c| c.path == p) {
            r.candidates.push(Candidate{path:p,scope:scope_for(&d,&root,&dir),reason:"filesystem-root ancestor walk; vendor docs leave root CLAUDE.md + .claude/CLAUDE.md coexistence underspecified".into(),state:State::Selected});
        }
    }
    r
}
fn codex(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let ch = cfg
        .codex_home
        .clone()
        .unwrap_or_else(|| cfg.home.join(".codex"));
    let mut r = base(
        Agent::Codex,
        dir.clone(),
        format!("git root: {}", root.display()),
        &[
            "$CODEX_HOME/AGENTS.override.md | AGENTS.md",
            "AGENTS.override.md | AGENTS.md",
        ],
    );
    add_global_first(
        &mut r,
        &ch,
        &["AGENTS.override.md", "AGENTS.md"],
        "global instruction location; first non-empty candidate",
        true,
    );
    for d in chain(&root, &dir) {
        add_first(
            &mut r,
            &d,
            (&root, &dir),
            &["AGENTS.override.md", "AGENTS.md"],
            "ancestor walk from git root to --dir; first non-empty candidate",
            false,
            true,
        );
    }
    r
}
fn pi(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = fs_root(&dir, cfg);
    let pd = cfg
        .pi_dir
        .clone()
        .unwrap_or_else(|| cfg.home.join(".pi/agent"));
    let mut r = base(
        Agent::Pi,
        dir.clone(),
        format!("filesystem root: {}", root.display()),
        &[
            "$PI_CODING_AGENT_DIR/{AGENTS.md, AGENTS.MD, CLAUDE.md, CLAUDE.MD}",
            "AGENTS.md | AGENTS.MD | CLAUDE.md | CLAUDE.MD",
        ],
    );
    add_global_first(
        &mut r,
        &pd,
        &["AGENTS.md", "AGENTS.MD", "CLAUDE.md", "CLAUDE.MD"],
        "global instruction location",
        false,
    );
    for d in chain(&root, &dir) {
        add_first(
            &mut r,
            &d,
            (&root, &dir),
            &["AGENTS.md", "AGENTS.MD", "CLAUDE.md", "CLAUDE.MD"],
            "filesystem-root ancestor walk; first candidate match in this directory",
            false,
            false,
        );
    }
    r
}
fn gemini(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut r = base(
        Agent::Gemini,
        dir.clone(),
        format!("git root: {}", root.display()),
        &["~/.gemini/GEMINI.md", "GEMINI.md"],
    );
    add_global(
        &mut r,
        cfg.home.join(".gemini/GEMINI.md"),
        "global instruction location",
    );
    for d in chain(&root, &dir) {
        add_all(
            &mut r,
            &d,
            &root,
            &dir,
            &[("GEMINI.md", false)],
            "ancestor walk from git root to --dir",
        );
    }
    r
}
fn amp(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = if dir.starts_with(&cfg.home) {
        cfg.home.clone()
    } else {
        fs_root(&dir, cfg)
    };
    let mut r = base(
        Agent::Amp,
        dir.clone(),
        format!(
            "{}: {}",
            if root == cfg.home {
                "home boundary"
            } else {
                "filesystem root"
            },
            root.display()
        ),
        &[
            "~/.config/amp/AGENTS.md",
            "AGENTS.md | AGENT.md | CLAUDE.md",
        ],
    );
    add_global(
        &mut r,
        cfg.home.join(".config/amp/AGENTS.md"),
        "global location",
    );
    for d in chain(&root, &dir) {
        add_first(
            &mut r,
            &d,
            (&root, &dir),
            &["AGENTS.md", "AGENT.md", "CLAUDE.md"],
            "ancestor walk from home boundary to --dir; first candidate match in this directory",
            false,
            false,
        );
    }
    r
}
fn goose(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut r = base(
        Agent::Goose,
        dir.clone(),
        format!("git root: {}", root.display()),
        &[
            "~/.config/goose/.goosehints + AGENTS.md",
            "AGENTS.md + .goosehints",
        ],
    );
    add_global(
        &mut r,
        cfg.home.join(".config/goose/.goosehints"),
        "global instruction location; composes",
    );
    add_global(
        &mut r,
        cfg.home.join(".config/goose/AGENTS.md"),
        "global instruction location; composes",
    );
    for d in chain(&root, &dir) {
        add_all(
            &mut r,
            &d,
            &root,
            &dir,
            &[("AGENTS.md", false), (".goosehints", false)],
            "ancestor walk from git root to --dir; files compose",
        );
    }
    r
}
fn qwen(dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut r = base(
        Agent::Qwen,
        dir.clone(),
        format!("git root: {}", root.display()),
        &[
            "~/.qwen/QWEN.md",
            "QWEN.md + AGENTS.md + .qwen/QWEN.local.md",
        ],
    );
    add_global(
        &mut r,
        cfg.home.join(".qwen/QWEN.md"),
        "global instruction location",
    );
    for d in chain(&root, &dir) {
        add_all(
            &mut r,
            &d,
            &root,
            &dir,
            &[("QWEN.md", false), ("AGENTS.md", false)],
            "ancestor walk from git root to --dir (source-observed)",
        );
    }
    let local = root.join(".qwen/QWEN.local.md");
    if exists(&local) {
        r.candidates.push(Candidate {
            path: local,
            scope: Scope::Local,
            reason: "project-root local memory, loaded last".into(),
            state: State::Selected,
        });
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    fn cfg(home: PathBuf, root: PathBuf) -> ResolverConfig {
        ResolverConfig {
            home,
            codex_home: None,
            pi_dir: None,
            fs_root: Some(root),
            opencode_disable_claude: false,
            opencode_disable_claude_prompt: false,
        }
    }
    #[test]
    fn opencode_shadows_fallback() {
        let t = tempdir().unwrap();
        let d = t.path().join("p");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(".git"), "").unwrap();
        fs::write(d.join("AGENTS.md"), "a").unwrap();
        fs::write(d.join("CLAUDE.md"), "c").unwrap();
        let r = resolve(
            Agent::OpenCode,
            d.clone(),
            &cfg(t.path().into(), t.path().into()),
        );
        assert_eq!(r.selected().count(), 1);
        assert!(r
            .candidates
            .iter()
            .any(|c| matches!(c.state, State::Shadowed { .. }) && c.path.ends_with("CLAUDE.md")));
    }
    #[test]
    fn opencode_env_disables_claude_fallbacks() {
        let t = tempdir().unwrap();
        let home = t.path().join("home");
        let d = t.path().join("p");
        fs::create_dir_all(home.join(".claude")).unwrap();
        fs::write(home.join(".claude/CLAUDE.md"), "g").unwrap();
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(".git"), "").unwrap();
        fs::write(d.join("CLAUDE.md"), "c").unwrap();
        let mut c = cfg(home, t.path().into());
        let r = resolve(Agent::OpenCode, d.clone(), &c);
        assert_eq!(r.selected().count(), 2);
        c.opencode_disable_claude_prompt = true;
        let r = resolve(Agent::OpenCode, d.clone(), &c);
        assert_eq!(r.selected().count(), 1); // project CLAUDE.md still applies
        c.opencode_disable_claude = true;
        let r = resolve(Agent::OpenCode, d, &c);
        assert_eq!(r.selected().count(), 0);
    }
    #[test]
    fn claude_composes_local() {
        let t = tempdir().unwrap();
        let d = t.path().join("p");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("CLAUDE.md"), "a").unwrap();
        fs::write(d.join("CLAUDE.local.md"), "b").unwrap();
        let r = resolve(Agent::Claude, d, &cfg(t.path().into(), t.path().into()));
        assert_eq!(r.selected().count(), 2);
    }
    #[test]
    fn codex_override_is_per_directory() {
        let t = tempdir().unwrap();
        let p = t.path().join("p");
        let d = p.join("child");
        fs::create_dir_all(&d).unwrap();
        fs::write(p.join(".git"), "").unwrap();
        fs::write(p.join("AGENTS.override.md"), "o").unwrap();
        fs::write(p.join("AGENTS.md"), "a").unwrap();
        fs::write(d.join("AGENTS.md"), "b").unwrap();
        let r = resolve(Agent::Codex, d, &cfg(t.path().into(), t.path().into()));
        assert_eq!(r.selected().count(), 2);
    }
    #[test]
    fn codex_nonempty_loser_reason() {
        let t = tempdir().unwrap();
        let d = t.path().join("p");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(".git"), "").unwrap();
        fs::write(d.join("AGENTS.override.md"), "o").unwrap();
        fs::write(d.join("AGENTS.md"), "a").unwrap();
        let r = resolve(
            Agent::Codex,
            d.clone(),
            &cfg(t.path().into(), t.path().into()),
        );
        let loser = r
            .candidates
            .iter()
            .find(|c| c.path.ends_with("p/AGENTS.md"))
            .unwrap();
        assert!(matches!(loser.state, State::Shadowed { .. }));
        assert_eq!(loser.reason, "lost same-directory candidate selection");
    }
    #[test]
    fn git_boundary_detected() {
        let t = tempdir().unwrap();
        let p = t.path().join("p");
        let d = p.join("a/b");
        fs::create_dir_all(&d).unwrap();
        fs::create_dir(p.join(".git")).unwrap();
        let r = resolve(Agent::Gemini, d, &cfg(t.path().into(), t.path().into()));
        assert!(r.boundary.contains(&p.display().to_string()));
    }
    #[test]
    fn pi_walk_is_injectable() {
        let t = tempdir().unwrap();
        let d = t.path().join("a/b");
        fs::create_dir_all(&d).unwrap();
        fs::write(t.path().join("AGENTS.md"), "r").unwrap();
        fs::write(d.join("AGENTS.md"), "d").unwrap();
        let r = resolve(Agent::Pi, d, &cfg(t.path().into(), t.path().into()));
        assert_eq!(r.selected().count(), 2);
    }
}
