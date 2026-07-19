mod amp;
mod claude;
mod codex;
mod gemini;
mod goose;
mod opencode;
mod pi;
mod qwen;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{Agent, Candidate, Resolution, ResolverConfig, Scope, State};

pub(super) enum Selection {
    FirstExisting,
    FirstNonEmpty,
}

pub(super) fn resolve(agent: Agent, dir: PathBuf, cfg: &ResolverConfig) -> Resolution {
    match agent {
        Agent::OpenCode => opencode::resolve(dir, cfg),
        Agent::Claude => claude::resolve(dir, cfg),
        Agent::Codex => codex::resolve(dir, cfg),
        Agent::Pi => pi::resolve(dir, cfg),
        Agent::Gemini => gemini::resolve(dir, cfg),
        Agent::Amp => amp::resolve(dir, cfg),
        Agent::Goose => goose::resolve(dir, cfg),
        Agent::Qwen => qwen::resolve(dir, cfg),
    }
}

pub(super) fn base(
    agent: Agent,
    dir: PathBuf,
    boundary: String,
    checked: Vec<String>,
) -> Resolution {
    Resolution {
        agent,
        dir,
        boundary,
        candidates: vec![],
        checked,
        notes: vec![],
    }
}

pub(super) fn names(names: &[&str]) -> String {
    names.join(" | ")
}
pub(super) fn exists(path: &Path) -> bool {
    path.is_file()
}
pub(super) fn scope_for(directory: &Path, root: &Path, _target: &Path) -> Scope {
    if directory == root {
        Scope::ProjectRoot
    } else {
        Scope::Directory
    }
}
pub(super) fn chain(root: &Path, dir: &Path) -> Vec<PathBuf> {
    let mut paths = vec![];
    let mut current = dir.to_path_buf();
    loop {
        paths.push(current.clone());
        if current == root || !current.pop() {
            break;
        }
    }
    paths.reverse();
    paths
}
pub(super) fn git_root(dir: &Path, floor: &Path) -> Option<PathBuf> {
    let mut current = dir.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if current == floor || !current.pop() {
            return None;
        }
    }
}
pub(super) fn fs_root(dir: &Path, cfg: &ResolverConfig) -> PathBuf {
    cfg.fs_root
        .clone()
        .unwrap_or_else(|| dir.ancestors().last().unwrap_or(dir).to_path_buf())
}
fn is_eligible(path: &Path, selection: &Selection) -> bool {
    matches!(selection, Selection::FirstExisting)
        || fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0)
}
pub(super) fn add_first<I>(
    resolution: &mut Resolution,
    paths: I,
    scope: Scope,
    reason: &str,
    selection: Selection,
) where
    I: IntoIterator<Item = PathBuf>,
{
    let found: Vec<_> = paths.into_iter().filter(|path| exists(path)).collect();
    let winner = found
        .iter()
        .find(|path| is_eligible(path, &selection))
        .cloned();
    for path in found {
        if Some(&path) == winner.as_ref() {
            resolution.candidates.push(Candidate {
                path,
                scope,
                reason: reason.into(),
                state: State::Selected,
            });
            continue;
        }
        let empty = fs::metadata(&path).map_or(true, |metadata| metadata.len() == 0);
        let reason = if matches!(selection, Selection::FirstNonEmpty) && empty {
            "existing but empty; skipped"
        } else {
            "lost candidate selection"
        };
        resolution.candidates.push(Candidate {
            path: path.clone(),
            scope,
            reason: reason.into(),
            state: State::Shadowed {
                by: winner.clone().unwrap_or(path),
            },
        });
    }
}
pub(super) fn add_all(
    resolution: &mut Resolution,
    directory: &Path,
    root: &Path,
    names: &[(&str, bool)],
    reason: &str,
) {
    for (name, local) in names {
        let path = directory.join(name);
        if exists(&path) {
            resolution.candidates.push(Candidate {
                path,
                scope: if *local {
                    Scope::Local
                } else {
                    scope_for(directory, root, directory)
                },
                reason: reason.into(),
                state: State::Selected,
            });
        }
    }
}
pub(super) fn add_global(resolution: &mut Resolution, path: PathBuf, reason: &str) {
    if exists(&path) {
        resolution.candidates.push(Candidate {
            path,
            scope: Scope::Global,
            reason: reason.into(),
            state: State::Selected,
        });
    }
}
pub(super) fn add_non_empty_global(resolution: &mut Resolution, path: PathBuf, reason: &str) {
    if fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
        resolution.candidates.push(Candidate {
            path,
            scope: Scope::Global,
            reason: reason.into(),
            state: State::Selected,
        });
    }
}
pub(super) fn add_excluded(resolution: &mut Resolution, path: PathBuf, scope: Scope, reason: &str) {
    if exists(&path) {
        resolution.candidates.push(Candidate {
            path,
            scope,
            reason: reason.into(),
            state: State::Excluded,
        });
    }
}

pub(super) fn add_shadowed(
    resolution: &mut Resolution,
    path: PathBuf,
    scope: Scope,
    by: PathBuf,
    reason: &str,
) {
    if exists(&path) {
        resolution.candidates.push(Candidate {
            path,
            scope,
            reason: reason.into(),
            state: State::Shadowed { by },
        });
    }
}
