use super::{add_all, add_global, base, chain, fs_root, git_root};
use crate::{Agent, ResolverConfig};
use std::path::PathBuf;
pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut resolution = base(
        Agent::Goose,
        dir.clone(),
        format!("git root: {}", root.display()),
        vec![
            "~/.config/goose/.goosehints + AGENTS.md".into(),
            "AGENTS.md + .goosehints".into(),
        ],
    );
    add_global(
        &mut resolution,
        cfg.home.join(".config/goose/.goosehints"),
        "global instruction location; composes",
    );
    add_global(
        &mut resolution,
        cfg.home.join(".config/goose/AGENTS.md"),
        "global instruction location; composes",
    );
    for directory in chain(&root, &dir) {
        add_all(
            &mut resolution,
            &directory,
            &root,
            &[("AGENTS.md", false), (".goosehints", false)],
            "ancestor walk from git root to --dir; files compose",
        );
    }
    resolution
}
