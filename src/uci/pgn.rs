use crate::bitboard::{file_char, file_of, rank_char, rank_of, square_to_algebraic};
use crate::board::{Board, Move, MoveType};
use crate::game::{Game, GameState, game::MoveError};
use crate::types::{Color, Piece, PieceType};

#[derive(Clone, Debug)]
pub struct PgnMetadata {
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
}

impl Default for PgnMetadata {
    fn default() -> Self {
        Self {
            event: "Engine Game".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "1".to_string(),
            white: "White".to_string(),
            black: "Black".to_string(),
        }
    }
}

pub fn game_to_pgn(finished_game: &Game, metadata: &PgnMetadata) -> Result<String, MoveError> {
    let mut replay_game = Game::from_fen(&finished_game.starting_fen).unwrap();

    let result = game_state_to_pgn_result(&finished_game.state);

    let mut output = String::new();

    push_pgn_tag(&mut output, "Event", &metadata.event);
    push_pgn_tag(&mut output, "Site", &metadata.site);
    push_pgn_tag(&mut output, "Date", &metadata.date);
    push_pgn_tag(&mut output, "Round", &metadata.round);
    push_pgn_tag(&mut output, "White", &metadata.white);
    push_pgn_tag(&mut output, "Black", &metadata.black);
    push_pgn_tag(&mut output, "Result", result);

    output.push('\n');

    for (index, &mv) in finished_game.move_history.iter().enumerate() {
        let side_to_move = replay_game.board.side_to_move;
        let san = play_move_as_san(&mut replay_game, mv)?;

        if side_to_move == Color::White {
            let move_number = index / 2 + 1;

            if index != 0 {
                output.push(' ');
            }

            output.push_str(&format!("{move_number}. {san}"));
        } else {
            output.push(' ');
            output.push_str(&san);
        }
    }

    if !finished_game.move_history.is_empty() {
        output.push(' ');
    }

    output.push_str(result);

    Ok(output)
}

fn game_state_to_pgn_result(state: &GameState) -> &'static str {
    match state {
        GameState::Checkmate {
            winner: Color::White,
        } => "1-0",

        GameState::Checkmate {
            winner: Color::Black,
        } => "0-1",

        GameState::Stalemate
        | GameState::DrawByRepetition
        | GameState::DrawByFiftyMoveRule
        | GameState::DrawByInsufficientMaterial => "1/2-1/2",

        GameState::Ongoing => "*",
    }
}

fn push_pgn_tag(output: &mut String, name: &str, value: &str) {
    output.push('[');
    output.push_str(name);
    output.push_str(" \"");
    output.push_str(&escape_pgn_tag_value(value));
    output.push_str("\"]\n");
}

fn escape_pgn_tag_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(ch),
        }
    }

    escaped
}

pub fn play_move_as_san(game: &mut Game, mv: Move) -> Result<String, MoveError> {
    let moving_piece = game
        .board
        .piece_at(mv.from)
        .expect("Move source square contains no piece");

    let mut san = String::new();

    if mv.kind == MoveType::Castle {
        if file_of(mv.to) > file_of(mv.from) {
            san.push_str("O-O");
        } else {
            san.push_str("O-O-O");
        }
    } else {
        let is_capture = is_capture_move(&game.board, mv);

        match moving_piece.kind {
            PieceType::Pawn => {
                if is_capture {
                    san.push(file_char(mv.from));
                    san.push('x');
                }

                san.push_str(&square_to_algebraic(mv.to));
            }

            PieceType::Knight
            | PieceType::Bishop
            | PieceType::Rook
            | PieceType::Queen
            | PieceType::King => {
                san.push(piece_letter(moving_piece.kind));

                if moving_piece.kind != PieceType::King {
                    san.push_str(&san_disambiguation(&mut game.board, mv, moving_piece));
                }

                if is_capture {
                    san.push('x');
                }

                san.push_str(&square_to_algebraic(mv.to));
            }
        }

        if let Some(promotion) = mv.promotion {
            san.push('=');
            san.push(promotion_letter(promotion));
        }
    }

    game.play_move(mv)?;

    match game.state {
        GameState::Checkmate { .. } => {
            san.push('#');
        }

        _ if game.board.in_check(game.board.side_to_move) => {
            san.push('+');
        }

        _ => {}
    }

    Ok(san)
}

fn san_disambiguation(board: &mut Board, mv: Move, moving_piece: Piece) -> String {
    let legal_moves = board.legal_moves(board.side_to_move);

    let mut same_file_conflict = false;
    let mut same_rank_conflict = false;
    let mut has_other_candidate = false;

    for &candidate in legal_moves.iter() {
        if candidate.from == mv.from || candidate.to != mv.to {
            continue;
        }

        let Some(candidate_piece) = board.piece_at(candidate.from) else {
            continue;
        };

        if candidate_piece.color != moving_piece.color || candidate_piece.kind != moving_piece.kind
        {
            continue;
        }

        has_other_candidate = true;

        if file_of(candidate.from) == file_of(mv.from) {
            same_file_conflict = true;
        }

        if rank_of(candidate.from) == rank_of(mv.from) {
            same_rank_conflict = true;
        }
    }

    if !has_other_candidate {
        return String::new();
    }

    if !same_file_conflict {
        return file_char(mv.from).to_string();
    }

    if !same_rank_conflict {
        return rank_char(mv.from).to_string();
    }

    square_to_algebraic(mv.from)
}

fn is_capture_move(board: &Board, mv: Move) -> bool {
    mv.kind == MoveType::Capture
        || mv.kind == MoveType::EnPassant
        || board.piece_at(mv.to).is_some()
}

fn piece_letter(piece: PieceType) -> char {
    match piece {
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook => 'R',
        PieceType::Queen => 'Q',
        PieceType::King => 'K',

        PieceType::Pawn => {
            panic!("Pawns do not have a SAN piece letter");
        }
    }
}

fn promotion_letter(piece: PieceType) -> char {
    match piece {
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook => 'R',
        PieceType::Queen => 'Q',

        PieceType::Pawn | PieceType::King => {
            panic!("Invalid promotion piece: {piece:?}");
        }
    }
}
