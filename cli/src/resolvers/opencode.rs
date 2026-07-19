use super::{
    add_excluded, add_first, add_global, add_shadowed, base, chain, exists, fs_root, git_root,
    names, scope_for, Selection,
};
use crate::{Agent, ResolverConfig, Scope};
use std::path::PathBuf;

pub(super) fn resolve(dir: PathBuf, cfg: &ResolverConfig) -> crate::Resolution {
    let disable_all = cfg.opencode.disable_claude;
    let disable_prompt = disable_all || cfg.opencode.disable_claude_prompt;
    let root = git_root(&dir, &fs_root(&dir, cfg));
    let project = root.clone().unwrap_or_else(|| dir.clone());
    let candidates = if disable_prompt {
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
            "excluded because OPENCODE_DISABLE_CLAUDE_CODE_PROMPT disables CLAUDE.md compatibility at every scope"
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
    if cfg.opencode.disable_project_config {
        for directory in chain(&project, &dir) {
            for name in ["AGENTS.md", "CLAUDE.md", "CONTEXT.md"] {
                add_excluded(&mut resolution, directory.join(name), scope_for(&directory, &project, &dir), "excluded because OPENCODE_DISABLE_PROJECT_CONFIG disables project instruction discovery");
            }
        }
        return resolution;
    }
    if disable_prompt {
        for directory in chain(&project, &dir) {
            add_excluded(
                &mut resolution,
                directory.join("CLAUDE.md"),
                scope_for(&directory, &project, &dir),
                "excluded because Claude Code prompt compatibility is disabled",
            );
        }
    }
    let directories = chain(&project, &dir);
    for (index, name) in candidates.iter().enumerate() {
        let matches: Vec<_> = directories
            .iter()
            .map(|directory| directory.join(name))
            .filter(|path| exists(path))
            .collect();
        if matches.is_empty() {
            continue;
        }
        let winner = matches[0].clone();
        for path in matches {
            let directory = path.parent().expect("instruction file has a parent");
            let scope = scope_for(directory, &project, &dir);
            let reason = if *name == "CONTEXT.md" {
                "ancestor walk from git root to --dir; first matching filename wins (deprecated)"
            } else {
                "ancestor walk from git root to --dir; first matching filename wins"
            };
            resolution.candidates.push(crate::Candidate {
                path,
                scope,
                reason: reason.into(),
                state: crate::State::Selected,
            });
        }
        for losing_name in candidates.iter().skip(index + 1) {
            for directory in &directories {
                add_shadowed(
                    &mut resolution,
                    directory.join(losing_name),
                    scope_for(directory, &project, &dir),
                    winner.clone(),
                    "shadowed because an earlier project instruction filename matched in the directory walk",
                );
            }
        }
        break;
    }
    resolution
}
