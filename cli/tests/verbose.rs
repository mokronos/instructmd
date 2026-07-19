use std::{fs, process::Command};

use tempfile::tempdir;

#[test]
fn verbose_explains_exclusions_before_layers_and_inclusions_below_headers() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("AGENTS.md"), "agents").unwrap();
    fs::write(project.join("CLAUDE.md"), "claude").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_instructmd"))
        .args(["opencode", "-v", "--no-content", "--no-color", "--dir"])
        .arg(&project)
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let exclusion = stdout.find("CLAUDE.md — excluded because").unwrap();
    let header = stdout.find("▌ [1] PROJECT ROOT").unwrap();
    let explanation = stdout.find("Why included:").unwrap();
    assert!(exclusion < header);
    assert!(header < explanation);
    assert!(stdout.contains("found this existing file while resolving PROJECT ROOT"));
    assert!(!stdout.contains("Shadowed candidates"));
}

#[test]
fn json_returns_structured_resolution_without_rendered_content() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join(".git"), "").unwrap();
    fs::write(project.join("GEMINI.md"), "not rendered").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_instructmd"))
        .args(["gemini", "--json", "--dir"])
        .arg(&project)
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["agent"], "gemini");
    assert_eq!(json["candidates"][0]["scope"], "project_root");
    assert!(!String::from_utf8(output.stdout)
        .unwrap()
        .contains("not rendered"));
}
