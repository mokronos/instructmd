use super::{
    add_excluded, add_first, add_global, base, chain, exists, fs_root, git_root, names, scope_for,
    Selection,
};
use crate::{Agent, ResolverConfig, Scope};
use std::path::PathBuf;

pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let disable_all = cfg.opencode.disable_claude;
    let disable_prompt = disable_all || cfg.opencode.disable_claude_prompt;
    let root = git_root(&dir, &fs_root(&dir, cfg));
    let project = root.clone().unwrap_or_else(|| dir.clone());
    let candidates = if disable_all {
        &["AGENTS.md", "CONTEXT.md"][..]
    } else {
        &["AGENTS.md", "CLAUDE.md", "CONTEXT.md"][..]
    };
    let mut resolution = base(
        Agent::OpenCode,
        dir.clone(),
        root.map(|path| format!("git root: {}", path.display()))
            .unwrap_or_else(|| "no git root; project directory only".into()),
        vec![
            "~/.config/opencode/AGENTS.md".into(),
            "~/.claude/CLAUDE.md (unless disabled)".into(),
            names(candidates),
        ],
    );
    if disable_prompt {
        add_global(
            &mut resolution,
            cfg.home.join(".config/opencode/AGENTS.md"),
            "global instruction location; ~/.claude/CLAUDE.md fallback disabled by environment",
        );
        let reason = if disable_all {
            "excluded because OPENCODE_DISABLE_CLAUDE_CODE disables CLAUDE.md compatibility at every scope"
        } else {
            "excluded because OPENCODE_DISABLE_CLAUDE_CODE_PROMPT disables only the global ~/.claude/CLAUDE.md compatibility prompt"
        };
        add_excluded(
            &mut resolution,
            cfg.home.join(".claude/CLAUDE.md"),
            Scope::Global,
            reason,
        );
    } else {
        add_first(
            &mut resolution,
            [
                cfg.home.join(".config/opencode/AGENTS.md"),
                cfg.home.join(".claude/CLAUDE.md"),
            ],
            Scope::Global,
            "global instruction location",
            Selection::FirstExisting,
        );
    }
    for directory in chain(&project, &dir) {
        if disable_all {
            add_excluded(&mut resolution, directory.join("CLAUDE.md"), scope_for(&directory, &project, &dir), "excluded because OPENCODE_DISABLE_CLAUDE_CODE disables CLAUDE.md compatibility at every scope");
        }
        let context_wins = exists(&directory.join("CONTEXT.md"))
            && !exists(&directory.join("AGENTS.md"))
            && (disable_all || !exists(&directory.join("CLAUDE.md")));
        let reason = if context_wins {
            "ancestor walk from git root to --dir; first candidate match in this directory (deprecated)"
        } else {
            "ancestor walk from git root to --dir; first candidate match in this directory"
        };
        add_first(
            &mut resolution,
            candidates.iter().map(|name| directory.join(name)),
            scope_for(&directory, &project, &dir),
            reason,
            Selection::FirstExisting,
        );
    }
    resolution
}
