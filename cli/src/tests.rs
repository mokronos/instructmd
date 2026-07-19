use std::{fs, path::PathBuf};

use tempfile::tempdir;

use crate::{resolve, Agent, CodexConfig, OpenCodeConfig, PiConfig, ResolverConfig, Scope, State};

fn config(home: PathBuf, root: PathBuf) -> ResolverConfig {
    ResolverConfig {
        home,
        fs_root: Some(root),
        codex: CodexConfig { home: None },
        pi: PiConfig { directory: None },
        opencode: OpenCodeConfig {
            disable_claude: false,
            disable_claude_prompt: false,
            disable_project_config: false,
        },
    }
}

#[test]
fn opencode_shadows_fallback() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("AGENTS.md"), "agents").unwrap();
    fs::write(project.join("CLAUDE.md"), "claude").unwrap();
    let resolution = resolve(
        Agent::OpenCode,
        project,
        &config(temp.path().into(), temp.path().into()),
    );
    assert_eq!(resolution.selected().count(), 1);
    assert!(resolution.candidates.iter().any(|candidate| matches!(
        candidate.state,
        State::Shadowed { .. }
    ) && candidate
        .path
        .ends_with("CLAUDE.md")));
}

#[test]
fn opencode_environment_switches_exclude_the_documented_claude_scopes() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(home.join(".claude/CLAUDE.md"), "global").unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("CLAUDE.md"), "project").unwrap();
    let mut config = config(home, temp.path().into());

    config.opencode.disable_claude_prompt = true;
    let prompt_disabled = resolve(Agent::OpenCode, project.clone(), &config);
    assert_eq!(prompt_disabled.selected().count(), 0);
    assert_eq!(prompt_disabled.excluded().count(), 2);

    config.opencode.disable_claude = true;
    let fully_disabled = resolve(Agent::OpenCode, project, &config);
    assert_eq!(fully_disabled.selected().count(), 0);
    assert_eq!(fully_disabled.excluded().count(), 2);
}

#[test]
fn opencode_uses_one_project_filename_for_the_entire_walk() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    let child = project.join("child");
    fs::create_dir_all(&child).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("CLAUDE.md"), "parent fallback").unwrap();
    fs::write(child.join("AGENTS.md"), "child native").unwrap();

    let resolution = resolve(
        Agent::OpenCode,
        child,
        &config(temp.path().into(), temp.path().into()),
    );

    assert_eq!(resolution.selected().count(), 1);
    assert!(resolution
        .selected()
        .all(|candidate| candidate.path.ends_with("AGENTS.md")));
    assert!(resolution
        .excluded()
        .any(|candidate| candidate.path.ends_with("CLAUDE.md")));
}

#[test]
fn opencode_can_disable_the_entire_project_walk() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("AGENTS.md"), "project").unwrap();
    let mut config = config(temp.path().into(), temp.path().into());
    config.opencode.disable_project_config = true;

    let resolution = resolve(Agent::OpenCode, project, &config);

    assert_eq!(resolution.selected().count(), 0);
    assert_eq!(resolution.excluded().count(), 1);
}

#[test]
fn pi_uses_the_configured_filesystem_root() {
    let temp = tempdir().unwrap();
    let directory = temp.path().join("a/b");
    fs::create_dir_all(&directory).unwrap();
    fs::write(temp.path().join("AGENTS.md"), "root").unwrap();
    fs::write(directory.join("AGENTS.md"), "directory").unwrap();
    let resolution = resolve(
        Agent::Pi,
        directory,
        &config(temp.path().into(), temp.path().into()),
    );
    assert_eq!(resolution.selected().count(), 2);
}

#[test]
fn pi_deduplicates_a_global_file_that_is_also_on_the_directory_chain() {
    let temp = tempdir().unwrap();
    let agent_directory = temp.path().join("agent");
    let directory = agent_directory.join("project");
    fs::create_dir_all(&directory).unwrap();
    fs::write(agent_directory.join("AGENTS.md"), "global").unwrap();
    let mut config = config(temp.path().into(), temp.path().into());
    config.pi.directory = Some(agent_directory);

    let resolution = resolve(Agent::Pi, directory, &config);

    assert_eq!(resolution.selected().count(), 1);
}

#[test]
fn codex_skips_empty_candidates_and_reports_a_note_for_large_projects() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("AGENTS.override.md"), "").unwrap();
    fs::write(project.join("AGENTS.md"), "x".repeat(32 * 1024 + 1)).unwrap();
    let resolution = resolve(
        Agent::Codex,
        project,
        &config(temp.path().into(), temp.path().into()),
    );
    assert_eq!(resolution.selected().count(), 1);
    assert_eq!(resolution.notes.len(), 1);
    assert!(resolution
        .candidates
        .iter()
        .any(|candidate| candidate.reason == "existing but empty; skipped"));
}

#[test]
fn claude_composes_local_files() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("CLAUDE.md"), "base").unwrap();
    fs::write(project.join("CLAUDE.local.md"), "local").unwrap();
    let resolution = resolve(
        Agent::Claude,
        project,
        &config(temp.path().into(), temp.path().into()),
    );
    assert_eq!(resolution.selected().count(), 2);
}

#[test]
fn gemini_composes_global_and_project_instructions() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".gemini")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(home.join(".gemini/GEMINI.md"), "global").unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("GEMINI.md"), "project").unwrap();
    let resolution = resolve(Agent::Gemini, project, &config(home, temp.path().into()));
    assert_eq!(resolution.selected().count(), 2);
    assert!(resolution
        .selected()
        .any(|candidate| candidate.scope == Scope::ProjectRoot));
}

#[test]
fn amp_stops_at_home_boundary() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = home.join("project/child");
    fs::create_dir_all(&project).unwrap();
    fs::write(home.join("AGENTS.md"), "home").unwrap();
    fs::write(home.join("project/AGENT.md"), "project").unwrap();
    let resolution = resolve(
        Agent::Amp,
        project,
        &config(home.clone(), temp.path().into()),
    );
    assert_eq!(resolution.selected().count(), 2);
    assert!(resolution.boundary.contains("home boundary"));
}

#[test]
fn goose_composes_both_global_and_project_candidates() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".config/goose")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(home.join(".config/goose/.goosehints"), "global").unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("AGENTS.md"), "agents").unwrap();
    fs::write(project.join(".goosehints"), "hints").unwrap();
    let resolution = resolve(Agent::Goose, project, &config(home, temp.path().into()));
    assert_eq!(resolution.selected().count(), 3);
}

#[test]
fn goose_loads_shared_agents_home_and_skips_empty_hints() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".agents")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(home.join(".agents/AGENTS.md"), "shared").unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join(".goosehints"), "").unwrap();

    let resolution = resolve(Agent::Goose, project, &config(home, temp.path().into()));

    assert_eq!(resolution.selected().count(), 1);
    assert!(resolution
        .selected()
        .next()
        .unwrap()
        .path
        .ends_with(".agents/AGENTS.md"));
}

#[test]
fn qwen_loads_project_root_local_memory_last() {
    let temp = tempdir().unwrap();
    let project = temp.path().join("project");
    let child = project.join("child");
    fs::create_dir_all(project.join(".qwen")).unwrap();
    fs::create_dir_all(&child).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("QWEN.md"), "instructions").unwrap();
    fs::write(project.join(".qwen/QWEN.local.md"), "memory").unwrap();
    let resolution = resolve(
        Agent::Qwen,
        child,
        &config(temp.path().into(), temp.path().into()),
    );
    let last = resolution.selected().last().unwrap();
    assert_eq!(last.scope, Scope::Local);
    assert!(last.path.ends_with(".qwen/QWEN.local.md"));
}

#[test]
fn qwen_loads_global_agents_and_requires_a_project_root_for_local_memory() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let directory = temp.path().join("directory");
    fs::create_dir_all(home.join(".qwen")).unwrap();
    fs::create_dir_all(directory.join(".qwen")).unwrap();
    fs::write(home.join(".qwen/AGENTS.md"), "global").unwrap();
    fs::write(directory.join(".qwen/QWEN.local.md"), "local").unwrap();

    let resolution = resolve(Agent::Qwen, directory, &config(home, temp.path().into()));

    assert_eq!(resolution.selected().count(), 1);
    assert!(resolution
        .selected()
        .next()
        .unwrap()
        .path
        .ends_with(".qwen/AGENTS.md"));
}
