use anstyle::{AnsiColor, Effects, Style};
use clap::Parser;
use instructmd::{resolve, Agent, ResolverConfig, Scope, State};
use std::{
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
};

#[derive(Parser)]
#[command(version, about = "Show coding-agent instruction-file startup layering")]
struct Cli {
    #[arg(value_enum, default_value_t = Agent::OpenCode)]
    agent: Agent,
    #[arg(long, default_value = ".")]
    dir: PathBuf,
    #[arg(long)]
    no_content: bool,
    #[arg(long)]
    no_color: bool,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let dir = cli.dir.canonicalize().map_err(|_| {
        format!(
            "directory does not exist or cannot be resolved: {}",
            cli.dir.display()
        )
    })?;
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()).into());
    };
    let r = resolve(cli.agent, dir, &ResolverConfig::from_env()?);
    print_resolution(
        &r,
        cli.no_content,
        !cli.no_color && io::stdout().is_terminal(),
    );
    Ok(())
}
fn paint(s: String, scope: Scope, color: bool) -> String {
    if !color {
        return s;
    }
    let shade = match scope {
        Scope::Global => AnsiColor::Magenta,
        Scope::ProjectRoot => AnsiColor::Blue,
        Scope::Directory => AnsiColor::Cyan,
        Scope::Local => AnsiColor::Yellow,
    };
    styled(s, Style::new().fg_color(Some(shade.into())), color)
}
fn styled(s: impl std::fmt::Display, style: Style, color: bool) -> String {
    if color {
        format!("{style}{s}{style:#}")
    } else {
        s.to_string()
    }
}
fn print_resolution(r: &instructmd::Resolution, no_content: bool, color: bool) {
    println!("{} — {} — {}", r.agent, r.dir.display(), r.boundary);
    let selected: Vec<_> = r.selected().collect();
    if selected.is_empty() {
        println!(
            "No startup instruction files found. Checked: {}.",
            r.checked.join("; ")
        );
    }
    for (i, c) in selected.iter().enumerate() {
        let h = format!(
            "━━ [{}] {} {} ━━━━━━━━━━━━",
            i + 1,
            c.scope,
            c.path.display()
        );
        println!("{}", paint(h, c.scope, color));
        let why = if color {
            styled("why:", Style::new().effects(Effects::DIMMED), true)
        } else {
            "why:".into()
        };
        println!("   {why} {}", c.reason);
        if !no_content {
            match fs::read_to_string(&c.path) {
                Ok(s) => {
                    let s = trim_trailing_blank_lines(&s);
                    if s.is_empty() {
                        println!("(empty file)")
                    } else {
                        println!("{s}")
                    }
                }
                Err(e) => println!("(could not read file: {e})"),
            }
        }
    }
    let shadows: Vec<_> = r
        .candidates
        .iter()
        .filter(|c| matches!(c.state, State::Shadowed { .. }))
        .collect();
    if !shadows.is_empty() {
        println!(
            "\n{}",
            if color {
                styled(
                    "Shadowed candidates",
                    Style::new()
                        .fg_color(Some(AnsiColor::Red.into()))
                        .effects(Effects::DIMMED),
                    true,
                )
            } else {
                "Shadowed candidates".into()
            }
        );
        for c in shadows {
            if let State::Shadowed { by } = &c.state {
                let loc = c
                    .path
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                if by == &c.path {
                    println!("- {} — skipped (empty) (in {})", c.path.display(), loc);
                } else {
                    println!(
                        "- {} — shadowed by {} (in {}) — {}",
                        c.path.display(),
                        by.display(),
                        loc,
                        c.reason
                    );
                }
            }
        }
    }
    println!("\nStartup resolution only — lazy/conditional loading not simulated.");
    if r.agent == Agent::Codex {
        let bytes: u64 = selected
            .iter()
            .filter(|c| c.scope != Scope::Global)
            .filter_map(|c| fs::metadata(&c.path).ok())
            .map(|m| m.len())
            .sum();
        if bytes > 32768 {
            println!("Codex note: selected project instructions total {bytes} bytes; the default aggregate cap is 32 KiB (not simulated).");
        }
    }
}
fn trim_trailing_blank_lines(content: &str) -> String {
    let mut lines: Vec<&str> = content.lines().collect();
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}
