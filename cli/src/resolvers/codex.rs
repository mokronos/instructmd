use super::{add_first, base, chain, fs_root, git_root, names, scope_for, Selection};
use crate::{Agent, ResolverConfig, Scope};
use std::{fs, path::PathBuf};

const CANDIDATES: &[&str] = &["AGENTS.override.md", "AGENTS.md"];

pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let home = cfg
        .codex
        .home
        .clone()
        .unwrap_or_else(|| cfg.home.join(".codex"));
    let mut resolution = base(
        Agent::Codex,
        dir.clone(),
        format!("git root: {}", root.display()),
        vec![
            format!("$CODEX_HOME/{}", names(CANDIDATES)),
            names(CANDIDATES),
        ],
    );
    add_first(
        &mut resolution,
        CANDIDATES.iter().map(|name| home.join(name)),
        Scope::Global,
        "global instruction location; first non-empty candidate",
        Selection::FirstNonEmpty,
    );
    for directory in chain(&root, &dir) {
        add_first(
            &mut resolution,
            CANDIDATES.iter().map(|name| directory.join(name)),
            scope_for(&directory, &root, &dir),
            "ancestor walk from git root to --dir; first non-empty candidate",
            Selection::FirstNonEmpty,
        );
    }
    let bytes: u64 = resolution
        .selected()
        .filter(|candidate| candidate.scope != Scope::Global)
        .filter_map(|candidate| fs::metadata(&candidate.path).ok())
        .map(|metadata| metadata.len())
        .sum();
    if bytes > 32 * 1024 {
        resolution.notes.push(format!("Selected project instructions total {bytes} bytes; Codex's default aggregate cap is 32 KiB (not simulated)."));
    }
    resolution
}
