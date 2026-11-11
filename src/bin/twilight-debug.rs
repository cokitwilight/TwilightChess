fn main() {
    if let Err(error) = chess_final::debug_uci::run_stdio() {
        eprintln!("twilight-debug: {error}");
        std::process::exit(1);
    }
}
