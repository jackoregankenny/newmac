//! Render a tab to plain text without a real terminal — handy for eyeballing
//! layout in CI logs or over SSH.
//!
//!   cargo run --example preview -- packages   # or browse | theme | options | save
//!
//! Optional trailing `WxH` sets the size, e.g. `packages 120x40`.

use newmac_core::{Catalog, Selection};
use newmac_tui::app::{App, Key};
use newmac_tui::ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let which = args.first().map(String::as_str).unwrap_or("packages");
    let (w, h) = args
        .iter()
        .find_map(|a| a.split_once('x'))
        .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
        .unwrap_or((118, 38));

    let catalog = Catalog::embedded();
    let sel = Selection::from_defaults(&catalog);
    let mut app = App::new(catalog, sel, std::env::temp_dir().join("newmac.conf"));

    match which {
        "browse" => app.on_key(Key::Char('2')),
        "theme" => app.on_key(Key::Char('3')),
        "options" => app.on_key(Key::Char('4')),
        "save" => {
            app.on_key(Key::Char('/'));
            for ch in "xcode".chars() {
                app.on_key(Key::Char(ch));
            }
            app.on_key(Key::Char(' '));
            app.on_key(Key::Esc);
            app.on_key(Key::Char('5'));
        }
        "search" => {
            app.on_key(Key::Char('/'));
            for ch in "term".chars() {
                app.on_key(Key::Char(ch));
            }
        }
        _ => {}
    }
    app.normalize();

    let backend = TestBackend::new(w, h);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
    let buf = terminal.backend().buffer().clone();
    for y in 0..h {
        let mut line = String::new();
        for x in 0..w {
            line.push_str(buf.cell((x, y)).map(|c| c.symbol()).unwrap_or(" "));
        }
        println!("{}", line.trim_end());
    }
}
