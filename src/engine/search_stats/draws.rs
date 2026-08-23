use super::count;

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

        println!("Draw Returns");
        println!(
            "  {:<22} {:>14}",
            "Repetition:",
            count(self.repetition_returns)
        );
        println!(
            "  {:<22} {:>14}",
            "Fifty move:",
            count(self.fifty_move_returns)
        );
        println!(
            "  {:<22} {:>14}",
            "Insufficient:",
            count(self.insufficient_material_returns)
        );
    }
}

impl_counter_stats_ops!(
    DrawStats,
    repetition_returns,
    fifty_move_returns,
    insufficient_material_returns,
);
