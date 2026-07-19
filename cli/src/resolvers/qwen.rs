use super::{add_all, add_global, base, chain, exists, fs_root, git_root};
use crate::{Agent, Candidate, ResolverConfig, Scope, State};
use std::path::PathBuf;
pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut resolution = base(
        Agent::Qwen,
        dir.clone(),
        format!("git root: {}", root.display()),
        vec![
            "~/.qwen/QWEN.md".into(),
            "QWEN.md + AGENTS.md + .qwen/QWEN.local.md".into(),
        ],
    );
    add_global(
        &mut resolution,
        cfg.home.join(".qwen/QWEN.md"),
        "global instruction location",
    );
    for directory in chain(&root, &dir) {
        add_all(
            &mut resolution,
            &directory,
            &root,
            &[("QWEN.md", false), ("AGENTS.md", false)],
            "ancestor walk from git root to --dir (source-observed)",
        );
    }
    let local = root.join(".qwen/QWEN.local.md");
    if exists(&local) {
        resolution.candidates.push(Candidate {
            path: local,
            scope: Scope::Local,
            reason: "project-root local memory, loaded last".into(),
            state: State::Selected,
        });
    }
    resolution
}
