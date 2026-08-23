use std::ops::{AddAssign, Sub};

use crate::engine::tt::TableStats;

#[derive(Clone, Copy, Debug, Default)]
pub struct TranspositionTableStats {
    pub main: TableStats,
    pub quiescence: TableStats,
}

impl TranspositionTableStats {
    pub(super) fn print(&self) {
        if self.main.probes == 0 && self.quiescence.probes == 0 {
            return;
        }

        println!("Transposition Tables");
        println!("Global TT - Negamax");
        self.main.print_stats_indented("    ");
        println!("Global TT - Quiescence");
        self.quiescence.print_stats_indented("    ");
    }
}

impl Sub for TranspositionTableStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            main: self.main - rhs.main,
            quiescence: self.quiescence - rhs.quiescence,
        }
    }
}

impl AddAssign for TranspositionTableStats {
    fn add_assign(&mut self, rhs: Self) {
        self.main += rhs.main;
        self.quiescence += rhs.quiescence;
    }
}
