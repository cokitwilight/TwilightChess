use eframe::egui;

use chess_final::ui::ChessApp;

fn main() -> eframe::Result<()> {
    if std::env::args().skip(1).any(|argument| argument == "--uci") {
        if let Err(error) = chess_final::uci::run_uci_stdio() {
            eprintln!("UCI error: {error}");
        }

        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rust Chess")
            .with_inner_size([900.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Chess",
        options,
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}
