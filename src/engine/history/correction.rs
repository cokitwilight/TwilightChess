use crate::board::Board;

const CORRECTION_SIZE: usize = 32768; // This is just an experimental values. Other values could be 16384 or 65536 for example. Eventually move this into configs.
const INDEX_SIZE: usize = 2 * CORRECTION_SIZE; // This is indexed by color so 2 * Correction Size

const UPDATE_LIMIT: i32 = 256;
const CORRECTION_LIMIT: i32 = 1024;
const CORRECTION_SCALER: i32 = 10; // this determines how powerful correction can be. A scaler of 1 means the stored value in the table is the CentiPawn value used. 
//  10 would represent 1/10 of the tables actual value is the cp value. With a max limit of 1024(currently) this means 1024/10 = ~100 max eval bonus

#[derive(Clone, Debug)]
pub struct CorrectionHistory {
    table: Box<[i32]>, // indexed as table[pawn_hash & Coorection Size]
}

impl Default for CorrectionHistory {
    fn default() -> Self {
        Self {
            table: vec![0; INDEX_SIZE].into_boxed_slice(),
        }
    }
}

impl CorrectionHistory {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, board: &Board) -> i32 {
        let index = index(board);

        self.table[index] / CORRECTION_SCALER
    }

    pub fn update(&mut self, board: &Board, delta: i32, depth: u16) {
        let index = index(board);
        let bonus = (delta * depth as i32 / 8).clamp(-UPDATE_LIMIT, UPDATE_LIMIT);

        correction_update(&mut self.table[index], bonus);
    }
}

fn index(board: &Board) -> usize {
    let index = board.pawn_hash() as usize & (CORRECTION_SIZE - 1);

    board.side_to_move().idx() * CORRECTION_SIZE + index
}

#[inline]
fn correction_update(value: &mut i32, bonus: i32) {
    // bonus was already clamped to UPDATE LIMIT
    let current = *value;

    *value = current + bonus - current * bonus.abs() / CORRECTION_LIMIT;
}
