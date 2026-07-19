use super::{add_non_empty_global, base, chain, exists, fs_root, git_root, scope_for};
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
            "~/.agents/AGENTS.md".into(),
            ".goosehints + AGENTS.md".into(),
        ],
    );
    add_non_empty_global(
        &mut resolution,
        cfg.home.join(".config/goose/.goosehints"),
        "global instruction location; composes",
    );
    add_non_empty_global(
        &mut resolution,
        cfg.home.join(".config/goose/AGENTS.md"),
        "global instruction location; composes",
    );
    add_non_empty_global(
        &mut resolution,
        cfg.home.join(".agents/AGENTS.md"),
        "shared agents-home instruction location",
    );
    for directory in chain(&root, &dir) {
        for name in [".goosehints", "AGENTS.md"] {
            let path = directory.join(name);
            if exists(&path) && path.metadata().is_ok_and(|metadata| metadata.len() > 0) {
                resolution.candidates.push(crate::Candidate {
                    path,
                    scope: scope_for(&directory, &root, &dir),
                    reason: "ancestor walk from git root to --dir; non-empty files compose".into(),
                    state: crate::State::Selected,
                });
            }
        }
    }
    resolution
}
