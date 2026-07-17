use eframe::egui;

use crate::bitboard::Square;
use crate::board::{MoveList, MoveType};
use crate::game::Game;
use crate::types::{Color, Piece, PieceType};
use crate::ui::BoardOrientation;
use crate::ui::input::{screen_pos_to_square, square_to_rect};

#[derive(Debug, Clone, Copy)]
pub struct BoardColors {
    pub light_square: egui::Color32,
    pub dark_square: egui::Color32,

    pub light_square_text: egui::Color32,
    pub dark_square_text: egui::Color32,

    pub selected_square: egui::Color32,

    pub quiet_move_dot: egui::Color32,
    pub capture_ring: egui::Color32,

    pub border: egui::Color32,

    pub white_piece: egui::Color32,
    pub black_piece: egui::Color32,
}

pub const GREEN_BOARD: BoardColors = BoardColors {
    light_square: egui::Color32::from_rgb(238, 238, 210),
    dark_square: egui::Color32::from_rgb(118, 150, 86),

    light_square_text: egui::Color32::from_rgb(118, 150, 86),
    dark_square_text: egui::Color32::from_rgb(238, 238, 210),

    // Opaque yellow, not RGBA blended.
    // This makes the selected square look the same on light/dark squares.
    selected_square: egui::Color32::from_rgb(246, 202, 85),

    quiet_move_dot: egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
    capture_ring: egui::Color32::from_rgba_premultiplied(0, 0, 0, 150),

    border: egui::Color32::BLACK,

    white_piece: egui::Color32::WHITE,
    black_piece: egui::Color32::BLACK,
};

pub const WOOD_BOARD: BoardColors = BoardColors {
    light_square: egui::Color32::from_rgb(240, 217, 181),
    dark_square: egui::Color32::from_rgb(181, 136, 99),

    light_square_text: egui::Color32::from_rgb(181, 136, 99),
    dark_square_text: egui::Color32::from_rgb(240, 217, 181),

    // Opaque yellow selection color.
    selected_square: egui::Color32::from_rgb(246, 202, 85),

    quiet_move_dot: egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
    capture_ring: egui::Color32::from_rgba_premultiplied(0, 0, 0, 150),

    border: egui::Color32::BLACK,

    white_piece: egui::Color32::WHITE,
    black_piece: egui::Color32::BLACK,
};

// Change only this line to switch schemes:
pub const BOARD_COLORS: BoardColors = WOOD_BOARD;

const MOVE_DOT: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 120);

const CAPTURE_RING: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardAction {
    Square(Square),
    Promotion(PieceType),
    PromotionCancel,
}

#[derive(Debug, Clone, Copy)]
pub struct PromotionPicker {
    pub square: Square,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromotionMenuItem {
    Piece(PieceType),
    Cancel,
}

pub fn draw_board_sized(
    ui: &mut egui::Ui,
    game: &Game,
    orientation: BoardOrientation,
    selected_square: Option<Square>,
    selected_moves: &MoveList,
    promotion_picker: Option<PromotionPicker>,
    board_size: f32,
) -> Option<BoardAction> {
    let board_size = board_size.max(200.0);

    let (board_rect, response) =
        ui.allocate_exact_size(egui::vec2(board_size, board_size), egui::Sense::click());

    let painter = ui.painter_at(board_rect);

    draw_squares(&painter, board_rect, orientation);
    draw_selected_square(&painter, board_rect, orientation, selected_square);
    draw_pieces(&painter, board_rect, game, orientation);
    draw_move_markers(&painter, board_rect, orientation, selected_moves);
    draw_coordinates(&painter, board_rect, orientation);
    draw_border(&painter, board_rect);

    if let Some(picker) = promotion_picker {
        draw_promotion_picker(&painter, board_rect, orientation, picker);
    }

    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        if let Some(picker) = promotion_picker {
            // While picker is open, consume board clicks.
            return promotion_action_at(pos, board_rect, orientation, picker);
        }

        return screen_pos_to_square(pos, board_rect, orientation).map(BoardAction::Square);
    }

    None
}

fn draw_squares(painter: &egui::Painter, board_rect: egui::Rect, orientation: BoardOrientation) {
    for sq in 0u8..64 {
        let rect = square_to_rect(sq, board_rect, orientation);

        let file = sq % 8;
        let rank = sq / 8;

        // a1 is dark. With your mapping, (file + rank) % 2 == 0 is dark.
        let is_dark = (file + rank) % 2 == 0;

        let color = if is_dark {
            BOARD_COLORS.dark_square
        } else {
            BOARD_COLORS.light_square
        };

        painter.rect_filled(rect, 0.0, color);
    }
}

fn draw_pieces(
    painter: &egui::Painter,
    board_rect: egui::Rect,
    game: &Game,
    orientation: BoardOrientation,
) {
    let board = game.board();

    for sq in 0u8..64 {
        let Some(piece) = board.piece_at(sq) else {
            continue;
        };

        let rect = square_to_rect(sq, board_rect, orientation);
        let piece_text = piece_to_unicode(piece);

        let font_size = rect.height() * 0.72;

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            piece_text,
            egui::FontId::proportional(font_size),
            egui::Color32::BLACK,
        );
    }
}

fn draw_coordinates(
    painter: &egui::Painter,
    board_rect: egui::Rect,
    orientation: BoardOrientation,
) {
    let square_size = board_rect.width().min(board_rect.height()) / 8.0;
    let font_size = square_size * 0.18;

    for sq in 0u8..64 {
        let file = sq % 8;
        let rank = sq / 8;

        let rect = square_to_rect(sq, board_rect, orientation);

        let is_dark = (file + rank) % 2 == 0;

        let text_color = if is_dark {
            BOARD_COLORS.dark_square_text
        } else {
            BOARD_COLORS.light_square_text
        };

        // Draw rank labels on the left edge from the player's visual perspective.
        let should_draw_rank = match orientation {
            BoardOrientation::WhiteBottom => file == 0,
            BoardOrientation::BlackBottom => file == 7,
        };

        if should_draw_rank {
            let rank_char = char::from(b'1' + rank);

            painter.text(
                rect.left_top() + egui::vec2(4.0, 3.0),
                egui::Align2::LEFT_TOP,
                rank_char,
                egui::FontId::proportional(font_size),
                text_color,
            );
        }

        // Draw file labels along the bottom edge from the player's visual perspective.
        let should_draw_file = match orientation {
            BoardOrientation::WhiteBottom => rank == 0,
            BoardOrientation::BlackBottom => rank == 7,
        };

        if should_draw_file {
            let file_char = char::from(b'a' + file);

            painter.text(
                rect.right_bottom() - egui::vec2(4.0, 3.0),
                egui::Align2::RIGHT_BOTTOM,
                file_char,
                egui::FontId::proportional(font_size),
                text_color,
            );
        }
    }
}

fn draw_border(painter: &egui::Painter, board_rect: egui::Rect) {
    painter.rect_stroke(
        board_rect,
        0.0,
        egui::Stroke::new(2.0, BOARD_COLORS.border),
        egui::StrokeKind::Outside,
    );
}

fn draw_selected_square(
    painter: &egui::Painter,
    board_rect: egui::Rect,
    orientation: BoardOrientation,
    selected_square: Option<Square>,
) {
    let Some(sq) = selected_square else {
        return;
    };

    let rect = square_to_rect(sq, board_rect, orientation);
    painter.rect_filled(rect, 0.0, BOARD_COLORS.selected_square);
}

fn draw_move_markers(
    painter: &egui::Painter,
    board_rect: egui::Rect,
    orientation: BoardOrientation,
    moves: &MoveList,
) {
    for mv in moves.iter().copied() {
        let rect = square_to_rect(mv.to, board_rect, orientation);
        let center = rect.center();

        let is_capture = matches!(mv.kind, MoveType::Capture);

        if is_capture {
            // Hollow circle / ring for captures.
            // For normal captures this surrounds the enemy piece.
            // For en passant, the target square is empty, but the move is still a capture.
            let radius = rect.width() * 0.40;
            let stroke_width = rect.width() * 0.065;

            painter.circle_stroke(
                center,
                radius,
                egui::Stroke::new(stroke_width, CAPTURE_RING),
            );
        } else {
            // Small filled dot for quiet legal moves.
            let radius = rect.width() * 0.13;

            painter.circle_filled(center, radius, MOVE_DOT);
        }
    }
}

fn draw_promotion_picker(
    painter: &egui::Painter,
    board_rect: egui::Rect,
    orientation: BoardOrientation,
    picker: PromotionPicker,
) {
    for (rect, item) in promotion_item_rects(board_rect, orientation, picker) {
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(245, 245, 245));

        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.5, egui::Color32::BLACK),
            egui::StrokeKind::Outside,
        );

        match item {
            PromotionMenuItem::Piece(kind) => {
                let piece = Piece {
                    color: picker.color,
                    kind,
                };

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    piece_to_unicode(piece),
                    egui::FontId::proportional(rect.height() * 0.72),
                    egui::Color32::BLACK,
                );
            }

            PromotionMenuItem::Cancel => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "×",
                    egui::FontId::proportional(rect.height() * 0.58),
                    egui::Color32::BLACK,
                );
            }
        }
    }
}

fn promotion_action_at(
    pos: egui::Pos2,
    board_rect: egui::Rect,
    orientation: BoardOrientation,
    picker: PromotionPicker,
) -> Option<BoardAction> {
    for (rect, item) in promotion_item_rects(board_rect, orientation, picker) {
        if rect.contains(pos) {
            return match item {
                PromotionMenuItem::Piece(kind) => Some(BoardAction::Promotion(kind)),
                PromotionMenuItem::Cancel => Some(BoardAction::PromotionCancel),
            };
        }
    }

    None
}

fn promotion_item_rects(
    board_rect: egui::Rect,
    orientation: BoardOrientation,
    picker: PromotionPicker,
) -> Vec<(egui::Rect, PromotionMenuItem)> {
    let target_rect = square_to_rect(picker.square, board_rect, orientation);
    let item_size = target_rect.width().min(target_rect.height());

    let menu_items = [
        PromotionMenuItem::Piece(PieceType::Queen),
        PromotionMenuItem::Piece(PieceType::Knight),
        PromotionMenuItem::Piece(PieceType::Rook),
        PromotionMenuItem::Piece(PieceType::Bishop),
        PromotionMenuItem::Cancel,
    ];

    let menu_height = item_size * menu_items.len() as f32;

    let x = target_rect.left();

    // Prefer a chess.com-style vertical dropdown from the promotion square.
    // If it would run off the board, clamp it back inside the board.
    let mut y = target_rect.top();

    if y + menu_height > board_rect.bottom() {
        y = board_rect.bottom() - menu_height;
    }

    if y < board_rect.top() {
        y = board_rect.top();
    }

    menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let min = egui::pos2(x, y + i as f32 * item_size);
            let rect = egui::Rect::from_min_size(min, egui::vec2(item_size, item_size));

            (rect, *item)
        })
        .collect()
}

fn piece_to_unicode(piece: Piece) -> &'static str {
    match (piece.color, piece.kind) {
        (Color::White, PieceType::Pawn) => "♙",
        (Color::White, PieceType::Knight) => "♘",
        (Color::White, PieceType::Bishop) => "♗",
        (Color::White, PieceType::Rook) => "♖",
        (Color::White, PieceType::Queen) => "♕",
        (Color::White, PieceType::King) => "♔",

        (Color::Black, PieceType::Pawn) => "♟",
        (Color::Black, PieceType::Knight) => "♞",
        (Color::Black, PieceType::Bishop) => "♝",
        (Color::Black, PieceType::Rook) => "♜",
        (Color::Black, PieceType::Queen) => "♛",
        (Color::Black, PieceType::King) => "♚",
    }
}

#[allow(dead_code)]
fn piece_color(color: Color) -> egui::Color32 {
    match color {
        Color::White => BOARD_COLORS.white_piece,
        Color::Black => BOARD_COLORS.black_piece,
    }
}
