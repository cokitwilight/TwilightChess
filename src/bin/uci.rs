fn main() {
    if let Err(error) = chess_final::uci::run_uci_stdio() {
        eprintln!("UCI error: {error}");
        std::process::exit(1);
    }
}
