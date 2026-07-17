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
    /// Explain included layers and existing files excluded during resolution.
    #[arg(short, long, global = true)]
    verbose: bool,
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
        cli.verbose,
    );
    Ok(())
}
const PALETTE: [(AnsiColor, AnsiColor); 6] = [
    (AnsiColor::BrightMagenta, AnsiColor::Magenta),
    (AnsiColor::BrightCyan, AnsiColor::Cyan),
    (AnsiColor::BrightGreen, AnsiColor::Green),
    (AnsiColor::BrightYellow, AnsiColor::Yellow),
    (AnsiColor::BrightBlue, AnsiColor::Blue),
    (AnsiColor::BrightRed, AnsiColor::Red),
];
fn layer_styles(layer: usize) -> (Style, Style) {
    let (bright, normal) = PALETTE[layer % PALETTE.len()];
    (
        Style::new()
            .fg_color(Some(bright.into()))
            .effects(Effects::BOLD),
        Style::new().fg_color(Some(normal.into())),
    )
}
fn tilde(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && s.starts_with(&home) => s.replacen(&home, "~", 1),
        _ => s,
    }
}
fn styled(s: impl std::fmt::Display, style: Style, color: bool) -> String {
    if color {
        format!("{style}{s}{style:#}")
    } else {
        s.to_string()
    }
}
fn print_resolution(r: &instructmd::Resolution, no_content: bool, color: bool, verbose: bool) {
    let banner = styled(
        format!(
            " instructmd · {} · {} — {} ",
            r.agent,
            tilde(&r.dir),
            r.boundary
        ),
        Style::new().effects(Effects::INVERT | Effects::BOLD),
        color,
    );
    println!("{banner}");
    if verbose {
        let excluded: Vec<_> = r.excluded().collect();
        if excluded.is_empty() {
            println!("Excluded existing candidates: none.");
        } else {
            println!("Excluded existing candidates:");
            for c in excluded {
                match &c.state {
                    State::Shadowed { by } if by == &c.path => {
                        println!("- {} — excluded because it is empty", tilde(&c.path));
                    }
                    State::Shadowed { by } => {
                        println!(
                            "- {} — excluded because {} (selected: {})",
                            tilde(&c.path),
                            c.reason,
                            tilde(by)
                        );
                    }
                    State::Excluded => {
                        println!("- {} — {}", tilde(&c.path), c.reason);
                    }
                    State::Selected => {
                        unreachable!("excluded iterator cannot return selected candidates")
                    }
                }
            }
        }
    }
    let selected: Vec<_> = r.selected().collect();
    if selected.is_empty() {
        println!(
            "No startup instruction files found. Checked: {}.",
            r.checked.join("; ")
        );
    }
    for (i, c) in selected.iter().enumerate() {
        let (header, body) = layer_styles(i);
        println!();
        let h = if verbose {
            format!("▌ [{}] {} {}", i + 1, c.scope, tilde(&c.path))
        } else {
            format!(
                "▌ [{}] {} {} — {}",
                i + 1,
                c.scope,
                tilde(&c.path),
                c.reason
            )
        };
        println!("{}", styled(h, header, color));
        if verbose {
            println!(
                "{}",
                styled(
                    format!(
                        "  Why included: {} found this existing file while resolving {}. {}.",
                        r.agent, c.scope, c.reason
                    ),
                    body,
                    color
                )
            );
        }
        if !no_content {
            match fs::read_to_string(&c.path) {
                Ok(s) => {
                    let s = trim_trailing_blank_lines(&s);
                    if s.is_empty() {
                        println!("{}", styled("(empty file)", body, color))
                    } else {
                        println!("{}", styled(s, body, color))
                    }
                }
                Err(e) => println!(
                    "{}",
                    styled(format!("(could not read file: {e})"), body, color)
                ),
            }
        }
    }
    let shadows: Vec<_> = r
        .candidates
        .iter()
        .filter(|c| matches!(c.state, State::Shadowed { .. }))
        .collect();
    if !verbose && !shadows.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_is_accepted_before_or_after_the_agent() {
        let before = Cli::try_parse_from(["instructmd", "-v", "claude"]).unwrap();
        assert!(before.verbose);
        assert_eq!(before.agent, Agent::Claude);

        let after = Cli::try_parse_from(["instructmd", "claude", "--verbose"]).unwrap();
        assert!(after.verbose);
        assert_eq!(after.agent, Agent::Claude);
    }
}
