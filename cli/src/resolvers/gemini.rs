use super::{add_all, add_global, base, chain, fs_root, git_root};
use crate::{Agent, ResolverConfig};
use std::path::PathBuf;
pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let root = git_root(&dir, &fs_root(&dir, cfg)).unwrap_or_else(|| dir.clone());
    let mut resolution = base(
        Agent::Gemini,
        dir.clone(),
        format!("git root: {}", root.display()),
        vec!["~/.gemini/GEMINI.md".into(), "GEMINI.md".into()],
    );
    add_global(
        &mut resolution,
        cfg.home.join(".gemini/GEMINI.md"),
        "global instruction location",
    );
    for directory in chain(&root, &dir) {
        add_all(
            &mut resolution,
            &directory,
            &root,
            &[("GEMINI.md", false)],
            "ancestor walk from git root to --dir",
        );
    }
    resolution
}
