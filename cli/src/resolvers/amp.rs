use super::{add_first, add_global, base, chain, fs_root, names, scope_for, Selection};
use crate::{Agent, ResolverConfig};
use std::path::PathBuf;
const CANDIDATES: &[&str] = &["AGENTS.md", "AGENT.md", "CLAUDE.md"];
pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = if dir.starts_with(&cfg.home) {
        cfg.home.clone()
    } else {
        fs_root(&dir, cfg)
    };
    let boundary = if root == cfg.home {
        "home boundary"
    } else {
        "filesystem root"
    };
    let mut resolution = base(
        Agent::Amp,
        dir.clone(),
        format!("{boundary}: {}", root.display()),
        vec!["~/.config/amp/AGENTS.md".into(), names(CANDIDATES)],
    );
    add_global(
        &mut resolution,
        cfg.home.join(".config/amp/AGENTS.md"),
        "global location",
    );
    for directory in chain(&root, &dir) {
        add_first(
            &mut resolution,
            CANDIDATES.iter().map(|name| directory.join(name)),
            scope_for(&directory, &root, &dir),
            "ancestor walk from home boundary to --dir; first candidate match in this directory",
            Selection::FirstExisting,
        );
    }
    resolution
}
