mod game;
mod gui;

use game::GameSettings;
use gui::*;

fn main() {
    env_logger::Builder::default()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let options = eframe::NativeOptions::default();

    if let Err(err) = eframe::run_native(
        "Snake",
        options,
        Box::new(|_context| {
            Ok(Box::new(App {
                state: Gui::new(),
                settings: GameSettings::default(),
            }))
        }),
    ) {
        log::error!("{err}");
    }
}
