//! All drawing. Reads [`crate::app::App`], writes to a ratatui `Frame`, and
//! records the geometry of every clickable thing back into `app.ui` so mouse
//! hit-testing (in `app.rs`) can map a click to a row. Kept separate from state
//! so snapshot tests can render any App to a `TestBackend`.

use crate::app::{App, InstallOutcome, Pane, Screen, StartEntry, Tab};
use newmac_core::theme::{Rgb, Theme};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

fn c(rgb: Rgb) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}

/// Entry point — draw the whole screen.
pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = app.themes[app.theme_idx].clone();
    let accent = c(theme.accent);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(f.area());

    if app.screen == Screen::Start {
        header_bar(f, chunks[0], &theme, " newmac — pick a preset ");
        start_screen(f, app, chunks[1], &theme);
        let hint = "↑/↓ move · enter choose · ? help · q quit";
        status_line(f, chunks[2], &theme, &app.status, hint);
        if app.show_help {
            help_overlay(f, accent);
        }
        return;
    }

    tab_bar(f, app, chunks[0], &theme);

    match app.tab {
        Tab::Packages => packages(f, app, chunks[1], &theme),
        Tab::Browse => browse(f, app, chunks[1], &theme),
        Tab::Theme => theme_tab(f, app, chunks[1]),
        Tab::Options => options(f, app, chunks[1], &theme),
        Tab::Save => save_tab(f, app, chunks[1], &theme),
    }

    let hint = "q quit · ? help · b presets · ^S save";
    status_line(f, chunks[2], &theme, &app.status, hint);

    if let Some(prompt) = app.prompt.clone() {
        prompt_overlay(f, &prompt, accent);
    }
    if app.show_help {
        help_overlay(f, accent);
    }
}

/// The bottom status line: message on the left, dim hint on the right.
fn status_line(f: &mut Frame, area: Rect, t: &Theme, status: &str, hint: &str) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {status}"),
            Style::new().fg(c(t.subtext)),
        ))),
        area,
    );
    let hint_w = hint.len() as u16 + 1;
    if area.width > hint_w {
        let a = Rect {
            x: area.x + area.width - hint_w,
            y: area.y,
            width: hint_w,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Span::styled(hint, Style::new().fg(Color::DarkGray))),
            a,
        );
    }
}

/// A plain bordered header (used by the start screen; the picker uses `tab_bar`).
fn header_bar(f: &mut Frame, area: Rect, t: &Theme, title: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::DarkGray))
        .title(Span::styled(
            title.to_string(),
            Style::new().fg(c(t.accent)).bold(),
        ));
    f.render_widget(block, area);
}

/// The "Presets" gate: pick a flavour, Custom, or Keep current.
fn start_screen(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let accent = c(t.accent);
    let entries = app.start_entries();
    let rows: Vec<ListItem> = entries
        .iter()
        .map(|e| {
            let (mark, title, desc) = match e {
                StartEntry::Flavour(i) => {
                    let fl = &app.flavours[*i];
                    let star = if fl.id == "jack" { "★" } else { "◆" };
                    (star, fl.title.clone(), fl.desc.clone())
                }
                StartEntry::Custom => (
                    "＋",
                    "Custom — à la carte".to_string(),
                    "start from defaults, pick everything yourself".to_string(),
                ),
                StartEntry::KeepCurrent => (
                    "↩",
                    "Keep current".to_string(),
                    "reuse what's in your newmac.conf".to_string(),
                ),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {mark} "), Style::new().fg(accent)),
                Span::styled(format!("{title:<16}"), Style::new().fg(c(t.text)).bold()),
                Span::styled(format!("  {desc}"), Style::new().fg(c(t.subtext))),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(Span::styled(" Presets ", Style::new().fg(accent)))
        .title_bottom(Span::styled(
            " enter to choose · you can tweak everything after · add your own: flavours/<id>.toml ",
            Style::new().fg(Color::DarkGray),
        ));
    app.ui.start.rect = block.inner(area);
    app.ui.start.state.select(Some(app.start_idx));
    let list = List::new(rows)
        .block(block)
        .highlight_style(Style::new().bg(c(t.surface)));
    f.render_stateful_widget(list, area, &mut app.ui.start.state);
}

fn tab_bar(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let accent = c(t.accent);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::DarkGray))
        .title(Span::styled(
            " newmac — your Mac, your way ",
            Style::new().fg(accent).bold(),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    app.ui.tabs_rect = inner;
    app.ui.tab_spans.clear();
    let contrast = bg_contrast(t);
    let mut xcur = inner.x;
    let mut spans: Vec<Span> = Vec::new();
    for (i, tab) in Tab::ALL.iter().enumerate() {
        let label = format!(" {}·{} ", i + 1, tab.title());
        let w = label.chars().count() as u16;
        app.ui.tab_spans.push((xcur, xcur + w));
        let style = if *tab == app.tab {
            Style::new().fg(contrast).bg(accent).bold()
        } else {
            Style::new().fg(c(t.subtext))
        };
        spans.push(Span::styled(label, style));
        xcur += w;
    }
    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn bg_contrast(t: &Theme) -> Color {
    let a = t.accent;
    let lum = 0.299 * a.r as f32 + 0.587 * a.g as f32 + 0.114 * a.b as f32;
    if lum > 140.0 {
        Color::Rgb(20, 20, 30)
    } else {
        Color::White
    }
}

fn badge_spans(item: &newmac_core::Item, t: &Theme) -> Vec<Span<'static>> {
    use newmac_core::Flag;
    let mut spans = Vec::new();
    for f in &item.flags {
        let (txt, col) = match f {
            Flag::Paid => ("$", c(t.yellow)),
            Flag::Account => ("account", c(t.blue)),
            Flag::Large => ("large", c(t.red)),
            Flag::Appstore => ("App Store", c(t.accent2)),
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("[{txt}]"),
            Style::new().fg(col).add_modifier(Modifier::DIM),
        ));
    }
    spans
}

fn packages(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let cols = Layout::horizontal([Constraint::Length(30), Constraint::Min(10)]).split(area);
    let accent = c(t.accent);

    // --- Categories pane (data first, then borrow app.ui mutably) ---
    let cats = app.categories();
    let cat_items: Vec<ListItem> = cats
        .iter()
        .map(|choice| {
            let (on, total) = app.selected_in(choice);
            let name = match choice {
                None => "All".to_string(),
                Some(id) => app.catalog.category_title(id),
            };
            let count = Span::styled(
                format!("  {on}/{total}"),
                Style::new().fg(if on > 0 { c(t.green) } else { Color::DarkGray }),
            );
            ListItem::new(Line::from(vec![Span::raw(name), count]))
        })
        .collect();
    let cat_focus = app.pane == Pane::Categories && app.query.is_empty();
    let cat_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(cat_focus, accent))
        .title(" Categories ");
    app.ui.cat.rect = cat_block.inner(cols[0]);
    app.ui.cat.state.select(Some(app.cat_idx));
    let cat_list = List::new(cat_items).block(cat_block).highlight_style(
        Style::new().fg(accent).add_modifier(if cat_focus {
            Modifier::BOLD
        } else {
            Modifier::empty()
        }),
    );
    f.render_stateful_widget(cat_list, cols[0], &mut app.ui.cat.state);

    // --- Items pane ---
    let vis = app.visible_items();
    let query = app.query.clone();
    let rows: Vec<ListItem> = vis
        .iter()
        .map(|&idx| {
            let it = &app.catalog.items[idx];
            let on = app.sel.is_selected(&it.id);
            let mark = if on {
                Span::styled("●", Style::new().fg(c(t.green)))
            } else {
                Span::styled("○", Style::new().fg(Color::DarkGray))
            };
            let name = Span::styled(
                format!(" {:<18}", it.name),
                if on {
                    Style::new().fg(c(t.text)).bold()
                } else {
                    Style::new().fg(c(t.text))
                },
            );
            let kind = Span::styled(
                format!("{:<9}", it.kind.label()),
                Style::new().fg(Color::DarkGray),
            );
            let mut line = vec![mark, name, Span::raw(" "), kind];
            line.extend(badge_spans(it, t));
            line.push(Span::styled(
                format!("  {}", it.description),
                Style::new().fg(c(t.subtext)),
            ));
            ListItem::new(Line::from(line))
        })
        .collect();

    let title = if query.is_empty() {
        format!(" {} ", pane_title(app))
    } else {
        format!(" search: {}_  ({} matches) ", query, vis.len())
    };
    let items_focus = app.pane == Pane::Items || !query.is_empty();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(items_focus, accent))
        .title(Span::styled(title, Style::new().fg(accent)));
    app.ui.items.rect = block.inner(cols[1]);
    if vis.is_empty() {
        app.ui.items.state.select(None);
    } else {
        app.ui
            .items
            .state
            .select(Some(app.item_idx.min(vis.len() - 1)));
    }
    let list = List::new(rows)
        .block(block)
        .highlight_style(Style::new().bg(c(t.surface)));
    f.render_stateful_widget(list, cols[1], &mut app.ui.items.state);
}

fn border_style(focus: bool, accent: Color) -> Style {
    if focus {
        Style::new().fg(accent)
    } else {
        Style::new().fg(Color::DarkGray)
    }
}

fn pane_title(app: &App) -> String {
    let cats = app.categories();
    match cats.get(app.cat_idx).cloned().flatten() {
        Some(id) => app.catalog.category_title(&id),
        None => "All packages".to_string(),
    }
}

fn browse(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let accent = c(t.accent);
    let vis = app.visible_brew();
    let rows: Vec<ListItem> = vis
        .iter()
        .map(|&i| {
            let p = &app.brew[i];
            let added = app.brew_added(i);
            let mark = if added {
                Span::styled("✓", Style::new().fg(c(t.green)))
            } else {
                Span::styled("＋", Style::new().fg(Color::DarkGray))
            };
            let name = Span::styled(
                format!(" {:<24}", p.name),
                if added {
                    Style::new().fg(c(t.text)).bold()
                } else {
                    Style::new().fg(c(t.text))
                },
            );
            let kind = Span::styled(
                format!("{:<8}", p.kind.label()),
                Style::new().fg(Color::DarkGray),
            );
            let installs = match p.installs {
                Some(n) => {
                    Span::styled(format!("{:>12}  ", human(n)), Style::new().fg(c(t.accent2)))
                }
                None => Span::raw(""),
            };
            ListItem::new(Line::from(vec![
                mark,
                name,
                Span::raw(" "),
                kind,
                installs,
                Span::styled(p.desc.clone(), Style::new().fg(c(t.subtext))),
            ]))
        })
        .collect();

    let title = if app.brew_query.is_empty() {
        format!(" Popular Homebrew ({}) ", app.brew.len())
    } else {
        format!(" search: {}_  ({} matches) ", app.brew_query, vis.len())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(Span::styled(title, Style::new().fg(accent)))
        .title_bottom(Span::styled(
            " click/space add · c custom formula · C custom cask · r live refresh · / search ",
            Style::new().fg(Color::DarkGray),
        ));
    app.ui.brew.rect = block.inner(area);
    if vis.is_empty() {
        app.ui.brew.state.select(None);
    } else {
        app.ui
            .brew
            .state
            .select(Some(app.brew_idx.min(vis.len() - 1)));
    }
    let list = List::new(rows)
        .block(block)
        .highlight_style(Style::new().bg(c(t.surface)));
    f.render_stateful_widget(list, area, &mut app.ui.brew.state);
}

fn human(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn theme_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Length(26), Constraint::Min(20)]).split(area);
    let cur = app.themes[app.theme_idx].clone();
    let accent = c(cur.accent);

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|th| {
            let active = th.id == app.sel.theme;
            let mark = if active {
                Span::styled("●", Style::new().fg(c(th.accent)))
            } else {
                Span::styled("○", Style::new().fg(Color::DarkGray))
            };
            ListItem::new(Line::from(vec![mark, Span::raw(format!(" {}", th.title))]))
        })
        .collect();
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(" Theme ");
    app.ui.theme.rect = list_block.inner(cols[0]);
    app.ui.theme.state.select(Some(app.theme_idx));
    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::new().fg(accent).bold());
    f.render_stateful_widget(list, cols[0], &mut app.ui.theme.state);

    // Preview over the theme's own background.
    let bg = c(cur.base);
    let preview = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(Span::styled(
            format!(" {} — live preview ", cur.title),
            Style::new().fg(accent),
        ))
        .style(Style::new().bg(bg));
    let inner = preview.inner(cols[1]);
    f.render_widget(preview, cols[1]);

    let names = ["accent", "accent2", "red", "green", "yellow", "blue"];
    let sw = cur.swatches();
    let mut lines: Vec<Line> = vec![Line::raw("")];
    for (name, rgb) in names.iter().zip(sw.iter()) {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("        ", Style::new().bg(c(*rgb))),
            Span::styled(
                format!("  {:<8} #{:02x}{:02x}{:02x}", name, rgb.r, rgb.g, rgb.b),
                Style::new().fg(c(cur.text)),
            ),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("❯", Style::new().fg(c(cur.green))),
        Span::styled(" ~/newmac ", Style::new().fg(c(cur.accent2))),
        Span::styled("git:(", Style::new().fg(c(cur.subtext))),
        Span::styled("main", Style::new().fg(c(cur.red))),
        Span::styled(") ", Style::new().fg(c(cur.subtext))),
        Span::styled("cargo build", Style::new().fg(c(cur.text))),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("  Finished ", Style::new().fg(c(cur.green))),
        Span::styled("release ", Style::new().fg(c(cur.text))),
        Span::styled("in 4.2s", Style::new().fg(c(cur.subtext))),
    ]));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "  click or space applies this theme to terminals · bar · borders · prompt",
        Style::new().fg(c(cur.subtext)),
    )));
    f.render_widget(Paragraph::new(lines).style(Style::new().bg(bg)), inner);
}

fn options(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let accent = c(t.accent);
    let tg = app.sel.toggles;
    let rows = [
        (
            tg.ricing,
            "Tiling desktop configs",
            "AeroSpace + sketchybar + borders + Karabiner remap",
        ),
        (
            tg.macos_defaults,
            "macOS UX defaults",
            "keyboard, Finder, Dock, screenshots",
        ),
        (
            tg.power,
            "Battery / power tuning",
            "pmset tweaks (needs sudo)",
        ),
        (
            tg.schedule,
            "Weekly auto-updates",
            "LaunchAgent, Mondays 10:00",
        ),
        (
            tg.dock,
            "Arrange the Dock",
            "replace the Dock to match your selection",
        ),
    ];
    let items: Vec<ListItem> = rows
        .iter()
        .map(|(on, label, desc)| {
            let box_ = if *on {
                Span::styled("[x]", Style::new().fg(c(t.green)))
            } else {
                Span::styled("[ ]", Style::new().fg(Color::DarkGray))
            };
            ListItem::new(Line::from(vec![
                box_,
                Span::styled(format!("  {label:<26}"), Style::new().fg(c(t.text))),
                Span::styled(desc.to_string(), Style::new().fg(c(t.subtext))),
            ]))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(" Options — click / space toggles ")
        .title_bottom(Span::styled(
            " tiling configs only apply if you also picked AeroSpace/sketchybar ",
            Style::new().fg(Color::DarkGray),
        ));
    app.ui.options.rect = block.inner(area);
    app.ui.options.state.select(Some(app.option_idx));
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(c(t.surface)));
    f.render_stateful_widget(list, area, &mut app.ui.options.state);
}

fn save_tab(f: &mut Frame, app: &mut App, area: Rect, t: &Theme) {
    let accent = c(t.accent);
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "Selection summary",
            Style::new().fg(accent).bold(),
        )),
        Line::raw(""),
    ];

    let core = app.catalog.always_ids().len();
    lines.push(kv("Core (always)", &format!("{core} shell & CLI tools"), t));

    for cat in app.catalog.selectable_categories() {
        let (on, total) = app.selected_in(&Some(cat.id.clone()));
        if on == 0 {
            continue;
        }
        let names: Vec<&str> = app
            .catalog
            .items_in(&cat.id)
            .filter(|it| app.sel.is_selected(&it.id))
            .map(|it| it.name.as_str())
            .collect();
        lines.push(kv(
            &format!("{} ({}/{})", cat.title, on, total),
            &names.join(", "),
            t,
        ));
    }

    if !app.sel.extra_brew.is_empty() || !app.sel.extra_cask.is_empty() {
        let mut extras = app.sel.extra_brew.clone();
        extras.extend(app.sel.extra_cask.iter().map(|c| format!("{c} (cask)")));
        lines.push(kv("Custom Homebrew", &extras.join(", "), t));
    }

    lines.push(kv("Theme", &app.themes[app.theme_idx].title, t));
    let tg = app.sel.toggles;
    lines.push(kv(
        "Toggles",
        &format!(
            "ricing={} defaults={} power={} weekly={} dock={}",
            b(tg.ricing),
            b(tg.macos_defaults),
            b(tg.power),
            b(tg.schedule),
            b(tg.dock)
        ),
        t,
    ));

    let flagged = app.flagged_selected();
    if !flagged.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "Heads up:",
            Style::new().fg(c(t.yellow)).bold(),
        )));
        for it in flagged {
            let notes: Vec<&str> = it.flags.iter().map(|fl| fl.note()).collect();
            lines.push(Line::from(vec![
                Span::styled(format!("  {} — ", it.name), Style::new().fg(c(t.text))),
                Span::styled(notes.join("; "), Style::new().fg(c(t.subtext))),
            ]));
        }
    }

    lines.push(Line::raw(""));
    if app.saved {
        lines.push(Line::from(Span::styled(
            format!("✓ Saved {}", app.conf_path.display()),
            Style::new().fg(c(t.green)).bold(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("Press s to write {}", app.conf_path.display()),
            Style::new().fg(accent).bold(),
        )));
    }

    // Result of the last in-TUI install run.
    match app.last_install {
        Some(InstallOutcome::Ok) => lines.push(Line::from(Span::styled(
            "✓ Install finished — check the log above; restart your shell.",
            Style::new().fg(c(t.green)),
        ))),
        Some(InstallOutcome::Failed(code)) => lines.push(Line::from(Span::styled(
            format!("✗ Install exited {code} — see the log above."),
            Style::new().fg(c(t.red)),
        ))),
        Some(InstallOutcome::DryRun) => lines.push(Line::from(Span::styled(
            "Dry-run shown above — nothing was installed.",
            Style::new().fg(c(t.subtext)),
        ))),
        None => {}
    }

    // Actions.
    lines.push(Line::raw(""));
    if app.can_install() {
        lines.push(Line::from(vec![
            Span::styled("Actions:  ", Style::new().fg(c(t.subtext))),
            Span::styled("s", Style::new().fg(accent).bold()),
            Span::styled(" save    ", Style::new().fg(c(t.text))),
            Span::styled("i", Style::new().fg(accent).bold()),
            Span::styled(" install now    ", Style::new().fg(c(t.text))),
            Span::styled("d", Style::new().fg(accent).bold()),
            Span::styled(" dry-run preview", Style::new().fg(c(t.text))),
        ]));
        lines.push(Line::from(Span::styled(
            "  (install runs the real brew/curl steps with live output, then returns here)",
            Style::new().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Save, quit, then run:  newmac install   (--dry-run to preview)",
            Style::new().fg(c(t.subtext)),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(" Save & install ");
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn kv(k: &str, v: &str, t: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {k:<20} "), Style::new().fg(c(t.text)).bold()),
        Span::styled(v.to_string(), Style::new().fg(c(t.subtext))),
    ])
}

fn b(v: bool) -> u8 {
    v as u8
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn prompt_overlay(f: &mut Frame, prompt: &crate::app::Prompt, accent: Color) {
    let area = centered(f.area(), 60, 5);
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(format!(" {} ", prompt.label));
    let text = Line::from(vec![
        Span::raw("  "),
        Span::styled(&prompt.input, Style::new().fg(Color::White).bold()),
        Span::styled("_", Style::new().fg(accent)),
    ]);
    f.render_widget(
        Paragraph::new(vec![
            Line::raw(""),
            text,
            Line::raw(""),
            Line::from(Span::styled(
                "  Enter to add · Esc to cancel",
                Style::new().fg(Color::DarkGray),
            )),
        ])
        .block(block),
        area,
    );
}

fn help_overlay(f: &mut Frame, accent: Color) {
    let area = centered(f.area(), 58, 20);
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(
            "newmac picker — keys & mouse",
            Style::new().fg(accent).bold(),
        )),
        Line::raw(""),
        Line::raw("  1–5 / [ ] / Tab   switch tabs   (or click a tab)"),
        Line::raw("  ↑ ↓ / j k         move   (or scroll the wheel)"),
        Line::raw("  ← → / h l         Packages: switch pane"),
        Line::raw("  space / enter     toggle highlighted   (or click a row)"),
        Line::raw("  /                 search (fuzzy, global)"),
        Line::raw("  a / n             select / clear all shown"),
        Line::raw("  Browse: a add · c custom formula · C custom cask · r refresh"),
        Line::raw("  Save:   s save · i install now · d dry-run"),
        Line::raw("  ^S                save from anywhere"),
        Line::raw("  q / Esc           quit"),
        Line::raw(""),
        Line::from(Span::styled(
            "  press any key to close",
            Style::new().fg(Color::DarkGray),
        )),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(accent))
        .title(" Help ");
    f.render_widget(Paragraph::new(lines).block(block), area);
}
