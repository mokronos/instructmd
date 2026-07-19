use super::{add_all, add_global, base, chain, exists, fs_root, scope_for};
use crate::{Agent, Candidate, ResolverConfig, State};
use std::path::PathBuf;

pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = fs_root(&dir, cfg);
    let mut resolution = base(
        Agent::Claude,
        dir.clone(),
        format!("filesystem root: {}", root.display()),
        vec![
            "~/.claude/CLAUDE.md".into(),
            "CLAUDE.md + CLAUDE.local.md + .claude/CLAUDE.md".into(),
        ],
    );
    add_global(
        &mut resolution,
        cfg.home.join(".claude/CLAUDE.md"),
        "global instruction location",
    );
    for directory in chain(&root, &dir) {
        add_all(
            &mut resolution,
            &directory,
            &root,
            &[("CLAUDE.md", false), ("CLAUDE.local.md", true)],
            "filesystem-root ancestor walk; files compose",
        );
        let path = directory.join(".claude/CLAUDE.md");
        if exists(&path)
            && !resolution
                .candidates
                .iter()
                .any(|candidate| candidate.path == path)
        {
            resolution.candidates.push(Candidate { path, scope: scope_for(&directory, &root, &dir), reason: "filesystem-root ancestor walk; vendor docs leave root CLAUDE.md + .claude/CLAUDE.md coexistence underspecified".into(), state: State::Selected });
        }
    }
    resolution
}
