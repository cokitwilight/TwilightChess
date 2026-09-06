use std::collections::HashSet;

use chess_final::bitboard::square_to_algebraic;
use chess_final::board::{Board, Move, MoveList, MoveType};
use chess_final::moves::king::{
    legal_king_capture_moves, legal_king_moves, legal_king_quiet_moves,
};
use chess_final::moves::knight::{
    legal_knight_capture_moves, legal_knight_moves, legal_knight_quiet_moves,
};
use chess_final::moves::pawn::{
    legal_en_passant_moves, legal_pawn_capture_moves, legal_pawn_moves, legal_pawn_promotion_moves,
    legal_pawn_quiet_moves,
};
use chess_final::moves::sliders::{
    legal_bishop_capture_moves, legal_bishop_moves, legal_bishop_quiet_moves,
    legal_queen_capture_moves, legal_queen_moves, legal_queen_quiet_moves,
    legal_rook_capture_moves, legal_rook_moves, legal_rook_quiet_moves,
};
use chess_final::moves::{
    MoveGenInfo, all_legal_capture_moves, all_legal_moves, all_legal_quiet_moves,
};
use chess_final::types::Color;

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

#[derive(Default)]
struct ScanStats {
    positions: u64,
    moves: u64,
}

type PieceMoveGenerator = fn(&Board, Color, &MoveGenInfo, &mut MoveList);

fn move_key(mv: Move) -> (u8, u8, u8, u8) {
    (
        mv.from(),
        mv.to(),
        mv.kind() as u8,
        mv.promotion().map_or(u8::MAX, |piece| piece as u8),
    )
}

fn move_name(mv: Move) -> String {
    let promotion = mv
        .promotion()
        .map_or_else(String::new, |piece| format!("={piece:?}"));

    format!(
        "{}{} {:?}{}",
        square_to_algebraic(mv.from()),
        square_to_algebraic(mv.to()),
        mv.kind(),
        promotion
    )
}

fn list_names(moves: &MoveList) -> Vec<String> {
    moves.iter().copied().map(move_name).collect()
}

fn extend(target: &mut MoveList, source: &MoveList) {
    for &mv in source.iter() {
        target.push(mv);
    }
}

fn assert_no_duplicates(label: &str, moves: &MoveList, fen: &str) {
    let mut seen = HashSet::with_capacity(moves.len());

    for &mv in moves.iter() {
        assert!(
            seen.insert(move_key(mv)),
            "{label} generated duplicate move {} in position:\n{fen}\nall moves: {:?}",
            move_name(mv),
            list_names(moves)
        );
    }
}

fn assert_same_moves(label: &str, actual: &MoveList, expected: &MoveList, fen: &str) {
    assert_no_duplicates(&format!("{label} (actual)"), actual, fen);
    assert_no_duplicates(&format!("{label} (expected)"), expected, fen);

    let actual_keys: HashSet<_> = actual.iter().copied().map(move_key).collect();
    let expected_keys: HashSet<_> = expected.iter().copied().map(move_key).collect();

    let missing: Vec<_> = expected
        .iter()
        .copied()
        .filter(|mv| !actual_keys.contains(&move_key(*mv)))
        .map(move_name)
        .collect();
    let unexpected: Vec<_> = actual
        .iter()
        .copied()
        .filter(|mv| !expected_keys.contains(&move_key(*mv)))
        .map(move_name)
        .collect();

    assert_eq!(
        actual.len(),
        expected.len(),
        "{label} length mismatch in position:\n{fen}\nmissing: {missing:?}\nunexpected: {unexpected:?}"
    );
    assert!(
        missing.is_empty() && unexpected.is_empty(),
        "{label} membership mismatch in position:\n{fen}\nmissing: {missing:?}\nunexpected: {unexpected:?}"
    );
}

fn generate_subdivided_moves(board: &Board) -> (MoveList, MoveList) {
    let color = board.side_to_move();
    let info = MoveGenInfo::calculate(board, color);
    let mut captures = MoveList::new();
    let mut quiets = MoveList::new();

    legal_king_capture_moves(board, color, &mut captures);
    legal_king_quiet_moves(board, color, &mut quiets);

    // In double check only a king move can be legal.
    if info.checkers.count_ones() > 1 {
        return (captures, quiets);
    }

    legal_en_passant_moves(board, color, &info, &mut captures);
    legal_pawn_capture_moves(board, color, &info, &mut captures);
    legal_pawn_quiet_moves(board, color, &info, &mut quiets);

    // Promotions are one pawn subdivision containing both quiet and capture moves.
    // Split that subdivision by MoveType when building the two top-level partitions.
    let mut promotions = MoveList::new();
    legal_pawn_promotion_moves(board, color, &info, &mut promotions);
    for &mv in promotions.iter() {
        match mv.kind() {
            MoveType::Capture => captures.push(mv),
            MoveType::Normal => quiets.push(mv),
            kind => panic!("promotion generator produced unexpected move type {kind:?}"),
        }
    }

    legal_knight_capture_moves(board, color, &info, &mut captures);
    legal_knight_quiet_moves(board, color, &info, &mut quiets);
    legal_bishop_capture_moves(board, color, &info, &mut captures);
    legal_bishop_quiet_moves(board, color, &info, &mut quiets);
    legal_rook_capture_moves(board, color, &info, &mut captures);
    legal_rook_quiet_moves(board, color, &info, &mut quiets);
    legal_queen_capture_moves(board, color, &info, &mut captures);
    legal_queen_quiet_moves(board, color, &info, &mut quiets);

    (captures, quiets)
}

fn assert_piece_subdivisions(board: &Board, fen: &str) {
    let color = board.side_to_move();
    let info = MoveGenInfo::calculate(board, color);

    let mut full = MoveList::new();
    let mut captures = MoveList::new();
    let mut quiets = MoveList::new();

    legal_king_moves(board, color, &mut full);
    legal_king_capture_moves(board, color, &mut captures);
    legal_king_quiet_moves(board, color, &mut quiets);
    extend(&mut captures, &quiets);
    assert_same_moves("king capture + quiet partition", &captures, &full, fen);

    let piece_generators: [(
        &str,
        PieceMoveGenerator,
        PieceMoveGenerator,
        PieceMoveGenerator,
    ); 4] = [
        (
            "knight capture + quiet partition",
            legal_knight_moves,
            legal_knight_capture_moves,
            legal_knight_quiet_moves,
        ),
        (
            "bishop capture + quiet partition",
            legal_bishop_moves,
            legal_bishop_capture_moves,
            legal_bishop_quiet_moves,
        ),
        (
            "rook capture + quiet partition",
            legal_rook_moves,
            legal_rook_capture_moves,
            legal_rook_quiet_moves,
        ),
        (
            "queen capture + quiet partition",
            legal_queen_moves,
            legal_queen_capture_moves,
            legal_queen_quiet_moves,
        ),
    ];

    for (label, full_generator, capture_generator, quiet_generator) in piece_generators {
        full.clear();
        captures.clear();
        quiets.clear();
        full_generator(board, color, &info, &mut full);
        capture_generator(board, color, &info, &mut captures);
        quiet_generator(board, color, &info, &mut quiets);
        extend(&mut captures, &quiets);
        assert_same_moves(label, &captures, &full, fen);
    }

    full.clear();
    captures.clear();
    quiets.clear();
    let mut promotions = MoveList::new();
    legal_pawn_moves(board, color, &info, &mut full);
    legal_pawn_capture_moves(board, color, &info, &mut captures);
    legal_pawn_quiet_moves(board, color, &info, &mut quiets);
    legal_pawn_promotion_moves(board, color, &info, &mut promotions);
    extend(&mut captures, &quiets);
    extend(&mut captures, &promotions);
    assert_same_moves(
        "pawn capture + quiet + promotion partition",
        &captures,
        &full,
        fen,
    );

    let mut en_passant = MoveList::new();
    legal_en_passant_moves(board, color, &info, &mut en_passant);
    assert_no_duplicates("en-passant subdivision", &en_passant, fen);
    assert!(
        en_passant.iter().all(|mv| mv.kind() == MoveType::EnPassant),
        "en-passant subdivision emitted a non-en-passant move in position:\n{fen}"
    );
}

fn assert_position_partitions(board: &mut Board, fen: &str) -> MoveList {
    let color = board.side_to_move();
    let mut all_legal = MoveList::new();
    all_legal_moves(board, color, &mut all_legal);

    let (captures, quiets) = generate_subdivided_moves(board);

    assert!(
        captures
            .iter()
            .all(|mv| matches!(mv.kind(), MoveType::Capture | MoveType::EnPassant)),
        "capture partition emitted a quiet move in position:\n{fen}\ncaptures: {:?}",
        list_names(&captures)
    );
    assert!(
        quiets
            .iter()
            .all(|mv| matches!(mv.kind(), MoveType::Normal | MoveType::Castle)),
        "quiet partition emitted a capture in position:\n{fen}\nquiets: {:?}",
        list_names(&quiets)
    );

    assert_no_duplicates("all legal moves", &all_legal, fen);
    assert_no_duplicates("capture partition", &captures, fen);
    assert_no_duplicates("quiet partition", &quiets, fen);

    let mut combined = captures;
    extend(&mut combined, &quiets);
    assert_same_moves(
        "capture + quiet partition versus all legal moves",
        &combined,
        &all_legal,
        fen,
    );

    let mut aggregate_captures = MoveList::new();
    all_legal_capture_moves(board, color, &mut aggregate_captures);
    assert!(
        aggregate_captures
            .iter()
            .all(|mv| matches!(mv.kind(), MoveType::Capture | MoveType::EnPassant)),
        "all_legal_capture_moves emitted a quiet move in position:\n{fen}\ncaptures: {:?}",
        list_names(&aggregate_captures)
    );
    assert_same_moves(
        "all_legal_capture_moves versus capture subdivisions",
        &aggregate_captures,
        &captures,
        fen,
    );

    let mut aggregate_quiets = MoveList::new();
    all_legal_quiet_moves(board, color, &mut aggregate_quiets);
    assert!(
        aggregate_quiets
            .iter()
            .all(|mv| matches!(mv.kind(), MoveType::Normal | MoveType::Castle)),
        "all_legal_quiet_moves emitted a capture in position:\n{fen}\nquiets: {:?}",
        list_names(&aggregate_quiets)
    );
    assert_same_moves(
        "all_legal_quiet_moves versus quiet subdivisions",
        &aggregate_quiets,
        &quiets,
        fen,
    );

    all_legal
}

fn scan_partitions(board: &mut Board, fen: &str, depth: usize, stats: &mut ScanStats) {
    board.assert_hash();
    let moves = assert_position_partitions(board, fen);
    stats.positions += 1;
    stats.moves += moves.len() as u64;

    if depth == 0 {
        return;
    }

    for &mv in moves.iter() {
        let undo = board.make_move(mv);
        let child_fen = format!("{fen}\npath ends with {}", move_name(mv));
        scan_partitions(board, &child_fen, depth - 1, stats);
        board.undo_move(undo);
        board.assert_hash();
    }
}

#[test]
fn piece_subdivisions_match_their_full_generators() {
    let positions = [
        STARTPOS,
        KIWIPETE,
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "k7/8/8/4KPpr/8/8/8/8 w - g6 0 1",
        "k3r3/8/8/8/1b6/8/8/4K3 w - - 0 1",
        "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/p7/1R2K3 b - - 0 1",
    ];

    for fen in positions {
        let board = Board::from_fen(fen).unwrap();
        assert_piece_subdivisions(&board, fen);
    }
}

#[test]
fn all_move_partitions_have_the_right_types_and_no_duplicates() {
    let positions = [
        STARTPOS,
        KIWIPETE,
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "k7/8/8/4KPpr/8/8/8/8 w - g6 0 1",
        "k3r3/8/8/8/1b6/8/8/4K3 w - - 0 1",
        "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/p7/1R2K3 b - - 0 1",
    ];

    for fen in positions {
        let mut board = Board::from_fen(fen).unwrap();
        assert_position_partitions(&mut board, fen);
    }
}

#[test]
fn kiwipete_recursive_partition_scan() {
    let mut board = Board::from_fen(KIWIPETE).unwrap();
    let mut stats = ScanStats::default();

    // Checks Kiwipete and every position reached within two plies (2,088 positions).
    scan_partitions(&mut board, KIWIPETE, 2, &mut stats);

    assert_eq!(stats.positions, 2_088);
    assert!(stats.moves > 0);
    board.assert_hash();
}

#[test]
#[ignore = "deep diagnostic: scans Kiwipete through depth 3 (about 100,000 positions)"]
fn kiwipete_deep_recursive_partition_scan() {
    let mut board = Board::from_fen(KIWIPETE).unwrap();
    let mut stats = ScanStats::default();

    scan_partitions(&mut board, KIWIPETE, 3, &mut stats);

    assert_eq!(stats.positions, 99_950);
    assert!(stats.moves > 0);
    board.assert_hash();
}
