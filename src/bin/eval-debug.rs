fn main() {
    if let Err(error) = chess_final::eval::debug::run_interactive() {
        eprintln!("Evaluation debugger error: {error}");
        std::process::exit(1);
    }
}
