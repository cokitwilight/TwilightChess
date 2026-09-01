use crate::engine::search_stats::formatting::{metric_count, section};

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawStats {
    pub repetition_returns: u64,
    pub fifty_move_returns: u64,
    pub insufficient_material_returns: u64,
}

impl DrawStats {
    pub(super) fn print(&self) {
        if self.repetition_returns == 0
            && self.fifty_move_returns == 0
            && self.insufficient_material_returns == 0
        {
            return;
        }

        section("Draw Returns");
        metric_count("Repetition", self.repetition_returns);
        metric_count("Fifty-move rule", self.fifty_move_returns);
        metric_count("Insufficient material", self.insufficient_material_returns);
    }
}

impl_counter_stats_ops!(
    DrawStats,
    repetition_returns,
    fifty_move_returns,
    insufficient_material_returns,
);
