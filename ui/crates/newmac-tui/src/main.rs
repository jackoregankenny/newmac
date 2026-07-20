//! `newmac-ui` — the Rust picker for newmac (ROADMAP #11, phase 1).
//!
//! Default: open the interactive ratatui picker, seeded from an existing
//! `newmac.conf` (or catalog defaults on a fresh machine), and write the conf
//! the bash `install.sh` consumes. Subcommands:
//!
//!   newmac-ui                     open the picker
//!   newmac-ui --conf <path>       use a specific conf (default: ./newmac.conf)
//!   newmac-ui catalog gen-sh      regenerate scripts/catalog.sh from catalog.toml
//!   newmac-ui brew refresh        print the live popular-Homebrew list
//!   newmac-ui --version | --help

use anyhow::{Context, Result};
use newmac_core::{Catalog, Selection};
use newmac_tui::app::{App, InstallOutcome, Key, Mouse};
use newmac_tui::{gensh, ui};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::DefaultTerminal;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = PickerOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("newmac-ui {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--conf" => {
                i += 1;
                opts.conf = Some(PathBuf::from(
                    args.get(i).context("--conf needs a path")?.clone(),
                ));
            }
            "--catalog" => {
                i += 1;
                opts.catalog = Some(PathBuf::from(
                    args.get(i).context("--catalog needs a path")?.clone(),
                ));
            }
            "--themes-dir" => {
                i += 1;
                opts.themes_dir = Some(PathBuf::from(
                    args.get(i).context("--themes-dir needs a path")?.clone(),
                ));
            }
            "catalog" => return cmd_catalog(&args[i + 1..]),
            "brew" => return cmd_brew(&args[i + 1..]),
            other => anyhow::bail!("unknown argument '{other}' (try --help)"),
        }
        i += 1;
    }

    run_picker(opts)
}

#[derive(Default)]
struct PickerOpts {
    conf: Option<PathBuf>,
    catalog: Option<PathBuf>,
    themes_dir: Option<PathBuf>,
}

fn print_help() {
    println!(
        "newmac-ui {} — searchable picker for newmac\n\n\
         USAGE:\n\
         \x20 newmac-ui [OPTIONS]           open the interactive picker\n\
         \x20 newmac-ui catalog gen-sh      regenerate scripts/catalog.sh from catalog.toml\n\
         \x20 newmac-ui brew refresh        print the live popular-Homebrew list\n\
         \x20 newmac-ui --version | --help\n\n\
         OPTIONS:\n\
         \x20 --conf <path>        conf to read/write (default: $NEWMAC/newmac.conf)\n\
         \x20 --catalog <path>     read this catalog.toml instead of the built-in one\n\
         \x20 --themes-dir <path>  read theme palettes from a config/themes dir\n",
        env!("CARGO_PKG_VERSION")
    );
}

/// Where the conf lives by default: `$NEWMAC/newmac.conf`, else `./newmac.conf`.
fn default_conf_path() -> PathBuf {
    if let Ok(dir) = std::env::var("NEWMAC") {
        return PathBuf::from(dir).join("newmac.conf");
    }
    PathBuf::from("newmac.conf")
}

fn load_selection(catalog: &Catalog, conf_path: &PathBuf) -> Selection {
    match std::fs::read_to_string(conf_path) {
        Ok(text) => {
            let mut s = Selection::parse_conf(&text);
            // Fresh conf with no selection line → seed from catalog defaults.
            if s.selected.is_empty() {
                s = Selection::from_defaults(catalog);
            }
            s
        }
        Err(_) => Selection::from_defaults(catalog),
    }
}

fn run_picker(opts: PickerOpts) -> Result<()> {
    let catalog = Catalog::from_path_or_embedded(opts.catalog.as_deref());
    let themes = newmac_core::theme::from_dir_or_embedded(opts.themes_dir.as_deref());
    let conf_path = opts.conf.unwrap_or_else(default_conf_path);
    let sel = load_selection(&catalog, &conf_path);
    let mut app = App::with_themes(catalog, sel, conf_path, themes);

    let mut terminal = enter_tui();
    let res = event_loop(&mut terminal, &mut app);
    leave_tui(terminal);
    res
}

/// Enter the alt-screen + raw mode and turn on mouse reporting.
fn enter_tui() -> DefaultTerminal {
    let terminal = ratatui::init();
    let _ = execute!(io::stdout(), EnableMouseCapture);
    terminal
}

/// Restore the terminal (mouse off, alt-screen off, cooked mode).
fn leave_tui(terminal: DefaultTerminal) {
    let _ = execute!(io::stdout(), DisableMouseCapture);
    drop(terminal);
    ratatui::restore();
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        app.normalize();
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(k) = translate_key(key) {
                        app.on_key(k);
                    }
                }
                Event::Mouse(m) => {
                    if let Some(mouse) = translate_mouse(m) {
                        app.on_mouse(mouse);
                    }
                }
                _ => {}
            }
        }

        // An install/dry-run was requested on the Save screen: drop out of the
        // TUI, run the bash installer with live output, then come back.
        if app.install_requested || app.dryrun_requested {
            let dry = app.dryrun_requested;
            app.install_requested = false;
            app.dryrun_requested = false;
            run_install_suspended(terminal, app, dry)?;
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Suspend the TUI, run `scripts/install.sh` inheriting the terminal so brew's
/// own progress shows live, then re-enter the picker with the outcome recorded.
fn run_install_suspended(terminal: &mut DefaultTerminal, app: &mut App, dry: bool) -> Result<()> {
    let Some(repo) = app.repo_dir.clone() else {
        app.status = "Can't locate the repo (set $NEWMAC) — save and run `newmac install`.".into();
        return Ok(());
    };
    let script = repo.join("scripts/install.sh");

    // Leave the alt-screen so the install prints to the normal scrollback.
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();

    println!(
        "\n=== newmac {} ===\n",
        if dry { "install --dry-run" } else { "install" }
    );
    let mut cmd = Command::new("bash");
    cmd.arg(&script).env("NEWMAC", &repo);
    if dry {
        cmd.arg("--dry-run");
    }
    let outcome = match cmd.status() {
        Ok(status) if dry => {
            let _ = status;
            InstallOutcome::DryRun
        }
        Ok(status) if status.success() => InstallOutcome::Ok,
        Ok(status) => InstallOutcome::Failed(status.code().unwrap_or(-1)),
        Err(e) => {
            println!("failed to launch {}: {e}", script.display());
            InstallOutcome::Failed(-1)
        }
    };
    app.last_install = Some(outcome);

    print!("\nPress Enter to return to the picker… ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);

    // Re-enter the TUI.
    *terminal = enter_tui();
    terminal.clear()?;
    app.status = match outcome {
        InstallOutcome::Ok => "Install finished.".into(),
        InstallOutcome::Failed(c) => format!("Install exited {c} — see the log."),
        InstallOutcome::DryRun => "Dry-run complete — nothing installed.".into(),
    };
    Ok(())
}

/// Map a crossterm mouse event onto the framework-agnostic [`Mouse`].
fn translate_mouse(ev: MouseEvent) -> Option<Mouse> {
    Some(match ev.kind {
        MouseEventKind::Down(MouseButton::Left) => Mouse::Down(ev.column, ev.row),
        MouseEventKind::ScrollDown => Mouse::Scroll(true, ev.column, ev.row),
        MouseEventKind::ScrollUp => Mouse::Scroll(false, ev.column, ev.row),
        _ => return None,
    })
}

/// Map a crossterm key event onto the framework-agnostic [`Key`].
fn translate_key(ev: KeyEvent) -> Option<Key> {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    Some(match ev.code {
        KeyCode::Char('s') if ctrl => Key::CtrlS,
        KeyCode::Char('c') if ctrl => Key::Esc, // Ctrl-C behaves like quit/cancel
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        _ => return None,
    })
}

// ---- Subcommands ------------------------------------------------------------

fn cmd_catalog(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("gen-sh") => {
            let catalog = Catalog::embedded();
            let body = gensh::render(&catalog);
            // Default output: scripts/catalog.sh relative to $NEWMAC or cwd.
            let out = if let Some(pos) = args.iter().position(|a| a == "--out") {
                PathBuf::from(args.get(pos + 1).context("--out needs a path")?)
            } else if args.iter().any(|a| a == "--stdout") {
                print!("{body}");
                return Ok(());
            } else {
                catalog_sh_path()
            };
            std::fs::write(&out, &body).with_context(|| format!("writing {}", out.display()))?;
            eprintln!("Wrote {} ({} items)", out.display(), catalog.items.len());
            Ok(())
        }
        _ => anyhow::bail!("usage: newmac-ui catalog gen-sh [--out <path> | --stdout]"),
    }
}

fn catalog_sh_path() -> PathBuf {
    if let Ok(dir) = std::env::var("NEWMAC") {
        return PathBuf::from(dir).join("scripts/catalog.sh");
    }
    PathBuf::from("scripts/catalog.sh")
}

fn cmd_brew(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("refresh") => {
            eprintln!("Fetching popular Homebrew packages from formulae.brew.sh…");
            let pkgs = newmac_core::brew::refresh(50)?;
            for p in &pkgs {
                let installs = p
                    .installs
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "-".into());
                println!("{:<8} {:>12}  {}", p.kind.label(), installs, p.name);
            }
            eprintln!("{} packages", pkgs.len());
            Ok(())
        }
        _ => anyhow::bail!("usage: newmac-ui brew refresh"),
    }
}
