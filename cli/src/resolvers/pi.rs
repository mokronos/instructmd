use super::{add_first, base, chain, fs_root, names, scope_for, Selection};
use crate::{Agent, ResolverConfig, Scope};
use std::path::PathBuf;
const CANDIDATES: &[&str] = &["AGENTS.md", "AGENTS.MD", "CLAUDE.md", "CLAUDE.MD"];
pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = fs_root(&dir, cfg);
    let home = cfg
        .pi
        .directory
        .clone()
        .unwrap_or_else(|| cfg.home.join(".pi/agent"));
    let mut resolution = base(
        Agent::Pi,
        dir.clone(),
        format!("filesystem root: {}", root.display()),
        vec![
            format!("$PI_CODING_AGENT_DIR/{{{}}}", CANDIDATES.join(", ")),
            names(CANDIDATES),
        ],
    );
    add_first(
        &mut resolution,
        CANDIDATES.iter().map(|name| home.join(name)),
        Scope::Global,
        "global instruction location",
        Selection::FirstExisting,
    );
    for directory in chain(&root, &dir) {
        if directory == home {
            continue;
        }
        add_first(
            &mut resolution,
            CANDIDATES.iter().map(|name| directory.join(name)),
            scope_for(&directory, &root, &dir),
            "filesystem-root ancestor walk; first candidate match in this directory",
            Selection::FirstExisting,
        );
    }
    resolution
}
