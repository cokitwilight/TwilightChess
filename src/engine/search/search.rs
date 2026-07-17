use crate::board::Board;
use crate::engine::{Engine, SearchContext, SearchLimits, SearchResult};

impl Engine {
    pub fn search(
        &mut self,
        board: &Board,
        limits: SearchLimits,
        repetition_history: &Vec<u64>,
    ) -> SearchResult {
        // CHECK IF CLONING HERE IS OK FOR REPETITION HISTORY, OR IF WE SHOULD PASS A REFERENCE
        let mut context = SearchContext::new(limits, repetition_history.clone());
        let mut board = board.clone();

        if let Some(book_mv) = self.get_book_move(&board) {
            // let piece = board.piece_at(book_mv.from).unwrap();

            println!("Book Move");
            // println!(
            //     "Book Move: {:?} {} to {}. End Stats: nodes={}, qnodes={}",
            //     piece.kind,
            //     square_name(book_mv.from),
            //     square_name(book_mv.to),
            //     self.nodes,
            //     self.qnodes
            // );

            return SearchResult {
                best_move: Some(book_mv),
                eval: 0,
                depth_reached: 0,
                stats: context.stats,
                pv: Vec::new(),
            };
        }

        // iterative deepening here

        let adjusted_depth = adjusted_depth_for_phase(context.limits.max_depth, board.phase());

        context.limits.max_depth = adjusted_depth;

        let search_result = self.iterative_deepening(&mut board, &mut context);

        SearchResult {
            best_move: search_result.best_move,
            eval: search_result.eval,
            depth_reached: search_result.depth_reached,
            stats: context.stats,
            pv: Vec::new(), // TODO: Implement principal variation
        }
    }
}

pub fn adjusted_depth_for_phase(base_depth: usize, phase: i32) -> usize {
    if phase <= 6 {
        base_depth + 2
    } else if phase <= 12 {
        base_depth + 1
    } else {
        base_depth
    }
}
