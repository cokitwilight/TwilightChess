use crate::bitboard::{Square, bishop_attacks, bit, pop_lsb, queen_attacks, rook_attacks};
use crate::board::{Board, Move, MoveList, MoveType};
use crate::moves::MoveGenInfo;
use crate::types::{Color, PieceType};

pub fn pseudo_bishop_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut bishops = board.pieces(color, PieceType::Bishop);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut bishops) {
        let targets = bishop_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn legal_bishop_moves(board: &Board, color: Color, moves: &mut MoveList, info: &MoveGenInfo) {
    let mut bishops = board.pieces(color, PieceType::Bishop);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut bishops) {
        let targets = bishop_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_bishop_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Bishop) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    let targets = bishop_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;
    let mut quiets = targets & empty;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }

    while let Some(to) = pop_lsb(&mut quiets) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Normal,
            promotion: None,
        });
    }
}

pub fn pseudo_bishop_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut bishops = board.pieces(color, PieceType::Bishop);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut bishops) {
        let targets = bishop_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}

pub fn legal_bishop_capture_moves(
    board: &Board,
    color: Color,
    moves: &mut MoveList,
    info: &MoveGenInfo,
) {
    let mut bishops = board.pieces(color, PieceType::Bishop);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut bishops) {
        let targets = bishop_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_bishop_capture_moves_at(
    board: &Board,
    color: Color,
    sq: Square,
    moves: &mut MoveList,
) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Bishop) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    let targets = bishop_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
}

pub fn pseudo_rook_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut rooks = board.pieces(color, PieceType::Rook);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut rooks) {
        let targets = rook_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn legal_rook_moves(board: &Board, color: Color, moves: &mut MoveList, info: &MoveGenInfo) {
    let mut rooks = board.pieces(color, PieceType::Rook);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut rooks) {
        let targets = rook_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_rook_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Rook) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    let targets = rook_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;
    let mut quiets = targets & empty;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }

    while let Some(to) = pop_lsb(&mut quiets) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Normal,
            promotion: None,
        });
    }
}

pub fn pseudo_rook_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut rooks = board.pieces(color, PieceType::Rook);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut rooks) {
        let targets = rook_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}

pub fn legal_rook_capture_moves(
    board: &Board,
    color: Color,
    moves: &mut MoveList,
    info: &MoveGenInfo,
) {
    let mut rooks = board.pieces(color, PieceType::Rook);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut rooks) {
        let targets = rook_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_rook_capture_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Rook) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    let targets = rook_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
}

pub fn pseudo_queen_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut queens = board.pieces(color, PieceType::Queen);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut queens) {
        let targets = queen_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn legal_queen_moves(board: &Board, color: Color, moves: &mut MoveList, info: &MoveGenInfo) {
    let mut queens = board.pieces(color, PieceType::Queen);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    while let Some(from) = pop_lsb(&mut queens) {
        let targets = queen_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;
        let mut quiets = targets & empty;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
        while let Some(to) = pop_lsb(&mut quiets) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Normal,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_queen_moves_at(board: &Board, color: Color, sq: Square, moves: &mut MoveList) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Queen) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();
    let empty = !occupancy;

    let targets = queen_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;
    let mut quiets = targets & empty;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }

    while let Some(to) = pop_lsb(&mut quiets) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Normal,
            promotion: None,
        });
    }
}

pub fn pseudo_queen_capture_moves(board: &Board, color: Color, moves: &mut MoveList) {
    let mut queens = board.pieces(color, PieceType::Queen);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut queens) {
        let targets = queen_attacks(from, occupancy) & !friends;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}
pub fn legal_queen_capture_moves(
    board: &Board,
    color: Color,
    moves: &mut MoveList,
    info: &MoveGenInfo,
) {
    let mut queens = board.pieces(color, PieceType::Queen);
    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    while let Some(from) = pop_lsb(&mut queens) {
        let targets = queen_attacks(from, occupancy)
            & !friends
            & info.pin_masks[from as usize]
            & info.check_mask;

        let mut captures = targets & enemies;

        while let Some(to) = pop_lsb(&mut captures) {
            moves.push(Move {
                from,
                to,
                kind: MoveType::Capture,
                promotion: None,
            });
        }
    }
}

pub fn pseudo_queen_capture_moves_at(
    board: &Board,
    color: Color,
    sq: Square,
    moves: &mut MoveList,
) {
    if sq >= 64 {
        return;
    }

    if board.pieces(color, PieceType::Queen) & bit(sq) == 0 {
        return;
    }

    let enemies = board.occupancy_of(color.opposite());
    let friends = board.occupancy_of(color);
    let occupancy = board.all_occupancy();

    let targets = queen_attacks(sq, occupancy) & !friends;

    let mut captures = targets & enemies;

    while let Some(to) = pop_lsb(&mut captures) {
        moves.push(Move {
            from: sq,
            to,
            kind: MoveType::Capture,
            promotion: None,
        });
    }
}
