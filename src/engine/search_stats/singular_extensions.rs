use super::{count, pct};

#[derive(Clone, Copy, Debug, Default)]
pub struct SingularExtensionStats {
    pub attempts: u64,
    pub extensions: u64,
    pub fail_highs: u64,
    pub no_alternatives: u64,
    pub verification_nodes: u64,
}

impl SingularExtensionStats {
    pub(super) fn print(&self, total_nodes: u64) {
        if self.attempts == 0
            && self.extensions == 0
            && self.fail_highs == 0
            && self.no_alternatives == 0
            && self.verification_nodes == 0
        {
            return;
        }

        println!("Singular Extensions");
        println!("  {:<22} {:>14}", "Attempts:", count(self.attempts));
        println!("  {:<22} {:>14}", "Extensions:", count(self.extensions));
        println!("  {:<22} {:>14}", "Alt fail-highs:", count(self.fail_highs));
        println!(
            "  {:<22} {:>14}",
            "No alternatives:",
            count(self.no_alternatives)
        );
        println!(
            "  {:<22} {:>14}",
            "Verification nodes:",
            count(self.verification_nodes)
        );

        if self.attempts > 0 {
            println!(
                "  {:<22} {:>14}",
                "Extension rate:",
                pct(self.extensions, self.attempts)
            );
            println!(
                "  {:<22} {:>14}",
                "Fail-high rate:",
                pct(self.fail_highs, self.attempts)
            );
            println!(
                "  {:<22} {:>14}",
                "No-alt rate:",
                pct(self.no_alternatives, self.attempts)
            );
            println!(
                "  {:<22} {:>14.2}",
                "Nodes / attempt:",
                self.verification_nodes as f64 / self.attempts as f64
            );
        }
        if total_nodes > 0 {
            println!(
                "  {:<22} {:>14}",
                "Verification share:",
                pct(self.verification_nodes, total_nodes)
            );
        }
    }
}

impl_counter_stats_ops!(
    SingularExtensionStats,
    attempts,
    extensions,
    fail_highs,
    no_alternatives,
    verification_nodes,
);
