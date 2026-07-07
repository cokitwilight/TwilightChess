use crate::board::{Board, Move};
use crate::search::tt::TTEntry;
use crate::search::{HistoryTable, KillerTable, TranspositionTable};

use num_format::{Locale, ToFormattedString};

#[derive(Clone, Debug)]
pub struct Engine {
    pub tt: TranspositionTable<TTEntry>,
    pub qtt: TranspositionTable<TTEntry>,
    // pub opening_book: OpeningBook,
    // Evaluator can be used for dynamic pst values or different evaluation values(favor aggressive play, favor defensive play, etc)
    //pub evaluator: Evaluator,
    pub history: HistoryTable,
    // pub killer_moves: KillerTable,
    // Later for options, such as search depth, time limit, elo rating, etc.
    // pub options: EngineOptions,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tt: TranspositionTable::new(64),  // 64 MB
            qtt: TranspositionTable::new(16), // 16 MB
            history: HistoryTable::new(),
            // killer_moves: KillerTable::default(),
        }
    }

    pub fn search(
        &mut self,
        board: &Board,
        limits: SearchLimits,
        repetition_history: &Vec<u64>,
    ) -> SearchResult {
        // CHECK IF CLONING HERE IS OK FOR REPETITION HISTORY, OR IF WE SHOULD PASS A REFERENCE
        let mut context = SearchContext::new(limits, repetition_history.clone());
        let mut board = board.clone();
        // iterative deepening here

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

#[derive(Clone, Debug)]
pub struct SearchContext {
    pub limits: SearchLimits,
    pub stats: SearchStats,

    pub killer_moves: KillerTable,
    pub repetition_history: Vec<u64>,

    pub start_time: std::time::Instant,
    pub stopped: bool,
}

impl SearchContext {
    pub fn new(limits: SearchLimits, repetition_history: Vec<u64>) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            killer_moves: KillerTable::new(),
            repetition_history,
            start_time: std::time::Instant::now(),
            stopped: false,
        }
    }

    pub fn should_stop(&self) -> bool {
        if self.stopped {
            return true;
        }

        if let Some(max_nodes) = self.limits.max_nodes {
            if self.stats.nodes >= max_nodes {
                return true;
            }
        }

        if let Some(time_limit_ms) = self.limits.time_limit_ms {
            if self.start_time.elapsed().as_millis() >= time_limit_ms as u128 {
                return true;
            }
        }

        false
    }
}

// used across searches to store information about the search, such as the best move found, the evaluation score, and the principal variation.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,

    pub tt: TableStats,
    pub qtt: TableStats,

    pub lmr_nodes: u64,
    pub lmr_researched: u64,

    pub null_moves: u64,
    pub null_cutoffs: u64,

    pub repetition_returns: u64,
    pub fifty_returns: u64,
}

impl SearchStats {
    pub fn print_all(&self, number: usize) {
        println!("{}. Stats:", number);
        self.print_nodes();
        self.print_pruning_heuristics();
        self.print_returns();
        self.print_tts();
        print!("\n");
    }
    pub fn print_nodes(&self) {
        println!(
            "Nodes: {}. Qnodes: {}",
            self.nodes.to_formatted_string(&Locale::en),
            self.qnodes.to_formatted_string(&Locale::en)
        );
    }
    pub fn print_pruning_heuristics(&self) {
        println!(
            "Lmr Nodes: {}. Researched Nodes: {}",
            self.lmr_nodes.to_formatted_string(&Locale::en),
            self.lmr_researched.to_formatted_string(&Locale::en)
        );
        println!(
            "Null Moves: {}. Null Prunes: {}",
            self.null_moves.to_formatted_string(&Locale::en),
            self.null_cutoffs.to_formatted_string(&Locale::en)
        );
    }
    pub fn print_tts(&self) {
        println!("Negamax Transposition Table:");
        self.tt.print_stats();
        println!("Quiescence Transposition Table:");
        self.qtt.print_stats();
    }
    pub fn print_returns(&self) {
        println!(
            "Repetition Returns: {}. Fifty Returns: {}",
            self.repetition_returns.to_formatted_string(&Locale::en),
            self.fifty_returns.to_formatted_string(&Locale::en)
        );
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TableStats {
    pub probes: u64,
    pub hits: u64,
    pub usable_hits: u64,
    pub exact_returns: u64,
    pub bound_cutoffs: u64,
}

impl TableStats {
    pub fn print_stats(&self) {
        println!("Probes: {}", self.probes.to_formatted_string(&Locale::en));
        println!(
            "Hits: {}. Usable Hits: {}",
            self.hits.to_formatted_string(&Locale::en),
            self.usable_hits.to_formatted_string(&Locale::en)
        );
        println!(
            "Exact Returns: {}. Bound cutoffs: {}",
            self.exact_returns.to_formatted_string(&Locale::en),
            self.bound_cutoffs.to_formatted_string(&Locale::en)
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SearchLimits {
    pub max_depth: usize,
    pub max_nodes: Option<u64>,
    pub time_limit_ms: Option<u64>,
}

impl SearchLimits {
    pub fn depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            max_nodes: None,
            time_limit_ms: None,
        }
    }

    pub fn depth_and_time(max_depth: usize, time_limit_ms: u64) -> Self {
        Self {
            max_depth,
            max_nodes: None,
            time_limit_ms: Some(time_limit_ms),
        }
    }
}

// what is actually returned from a search, including the best move found, the evaluation score, the depth reached, and the principal variation.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub eval: i32,
    pub depth_reached: usize,
    pub stats: SearchStats,
    pub pv: Vec<Move>,
}
