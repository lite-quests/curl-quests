mod app;
mod db;
mod quests;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = app::App::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        app.run(terminal)
    })
}
