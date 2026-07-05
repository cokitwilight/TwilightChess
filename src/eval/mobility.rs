use crate::{
    bitboard::{
        Bitboard, FILE_A, FILE_H, RANK_3, RANK_6,
        attacks::all_knight_attacks,
        king_attacks, knight_attacks, pop_lsb,
        rays::{all_bishop_attacks, all_queen_attacks, all_rook_attacks},
    },
    board::Board,
    types::{Color, PieceType},
};

pub fn mobility_score(board: &Board, phase: i32) -> i32 {
    mobility_score_raw(board, Color::White, phase) - mobility_score_raw(board, Color::Black, phase)
}

pub fn mobility_score_raw(board: &Board, color: Color, phase: i32) -> i32 {
    // check every move and subtract how many valid moves there are
    let mut score = 0;

    let friends = board.occupancy_of(color);

    score += pawn_moves_bitboard(board, color).count_ones() as i32;

    let occupied = board.all_occupancy();

    let pawn_attacks = pawn_capture_bitboard(board, color.opposite());
    let knight_and_bishop_attacks =
        all_knight_attacks(board.pieces(color.opposite(), PieceType::Knight))
            | all_bishop_attacks(board.pieces(color.opposite(), PieceType::Bishop), occupied);

    let rook_attacks = all_rook_attacks(board.pieces(color.opposite(), PieceType::Rook), occupied);

    let queen_attacks =
        all_queen_attacks(board.pieces(color.opposite(), PieceType::Queen), occupied);

    let available_knight_moves =
        all_knight_attacks(board.pieces(color, PieceType::Knight)) & !pawn_attacks & !friends;

    let available_bishop_moves =
        all_bishop_attacks(board.pieces(color, PieceType::Bishop), occupied)
            & !pawn_attacks
            & !friends;

    let available_rook_moves = all_rook_attacks(board.pieces(color, PieceType::Rook), occupied)
        & !(pawn_attacks | knight_and_bishop_attacks | friends);

    let available_queen_moves = all_queen_attacks(board.pieces(color, PieceType::Queen), occupied)
        & !(pawn_attacks | knight_and_bishop_attacks | rook_attacks | friends);

    let Some(king_sq) = pop_lsb(&mut board.pieces(color, PieceType::King)) else {
        panic!("No king in king bit board in mobility score!");
    };

    let safe_king_squares = (king_attacks(king_sq)
        & !(pawn_attacks | knight_and_bishop_attacks | rook_attacks | queen_attacks | friends))
        .count_ones() as i32;

    if safe_king_squares <= 1 {
        score -= 20;
    } else if safe_king_squares <= 4 {
        score += 5;
    } else {
        if phase <= 10 {
            score += 15;
        } else {
            score -= 5; // king is too open in middle game
        }
    }

    score += available_knight_moves.count_ones() as i32 * 4;
    score += available_bishop_moves.count_ones() as i32 * 3;
    score += available_rook_moves.count_ones() as i32 * 3;
    score += available_queen_moves.count_ones() as i32 * 2;

    score
}

// since not all pawn moves are captures. Ignore en passant for now as it might be too expensive/complicated
fn pawn_moves_bitboard(board: &Board, color: Color) -> Bitboard {
    let pawns = board.pieces(color, PieceType::Pawn);

    let empty = !board.all_occupancy();

    let mut moves = 0u64;

    match color {
        Color::White => {
            let single_pushes = (pawns << 8) & empty;
            moves |= single_pushes;

            moves |= ((single_pushes & RANK_3) << 8) & empty;
        }
        Color::Black => {
            let single_pushes = (pawns >> 8) & empty;
            moves |= single_pushes;

            moves |= ((single_pushes & RANK_6) >> 8) & empty;
        }
    }

    moves
}

fn pawn_capture_bitboard(board: &Board, color: Color) -> Bitboard {
    // includes friendly pieces since they are counted as defended
    let pawns = board.pieces(color, PieceType::Pawn);

    let mut moves = 0u64;

    match color {
        Color::White => {
            moves |= (pawns & !FILE_A) << 7;

            moves |= (pawns & !FILE_H) << 9;
        }
        Color::Black => {
            moves |= (pawns & !FILE_A) >> 9;

            moves |= (pawns & !FILE_H) >> 7;
        }
    }
    moves
}

fn knight_capture_biboard(board: &Board, color: Color) -> Bitboard {
    let mut knights = board.pieces(color, PieceType::Knight);

    let mut moves = 0u64;

    while let Some(sq) = pop_lsb(&mut knights) {
        moves |= knight_attacks(sq);
    }
    moves
}
