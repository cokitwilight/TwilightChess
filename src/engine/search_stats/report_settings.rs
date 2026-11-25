use std::io::{self, IsTerminal};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReportSection {
    Nodes,
    Iterative,
    Cutoffs,
    Pvs,
    Aspiration,
    Quiescence,
    Singular,
    Improvement,
    Estimated,
    Correction,
    Moves,
    MoveOrdering,
    History,
    Lmr,
    MateDistance,
    Razoring,
    Rfp,
    Lmp,
    Futility,
    NullMove,
    MainSee,
    Delta,
    QSee,
    TerminalNodes,
    Draws,
    Tt,
    All,
    Search,
    Evaluation,
    Ordering,
    Pruning,
    MainPruning,
    QPruning,
    Terminal,
}

impl ReportSection {
    pub fn tree(self) -> (&'static str, &'static [Self]) {
        use ReportSection::*;
        match self {
            All => (
                "All statistics",
                &[Search, Evaluation, Ordering, Pruning, Terminal, Tt],
            ),
            Search => (
                "Search",
                &[
                    Nodes, Iterative, Cutoffs, Pvs, Aspiration, Quiescence, Singular,
                ],
            ),
            Evaluation => ("Evaluation", &[Improvement, Estimated, Correction]),
            Ordering => ("Moves and history", &[Moves, MoveOrdering, History]),
            Pruning => ("Pruning", &[MainPruning, Lmr, QPruning]),
            MainPruning => (
                "Main-search pruning",
                &[
                    MateDistance,
                    Razoring,
                    Rfp,
                    Lmp,
                    Futility,
                    NullMove,
                    MainSee,
                ],
            ),
            QPruning => ("Quiescence pruning", &[Delta, QSee]),
            Terminal => ("Terminal positions", &[TerminalNodes, Draws]),
            Nodes => ("Nodes", &[]),
            Iterative => ("Iterative deepening", &[]),
            Cutoffs => ("Cutoffs", &[]),
            Pvs => ("Principal variation search", &[]),
            Aspiration => ("Aspiration windows", &[]),
            Quiescence => ("Quiescence search", &[]),
            Singular => ("Singular extensions", &[]),
            Improvement => ("Evaluation improvement", &[]),
            Estimated => ("Estimated score", &[]),
            Correction => ("Correction history", &[]),
            Moves => ("Moves", &[]),
            MoveOrdering => ("Move ordering", &[]),
            History => ("History updates", &[]),
            Lmr => ("Late move reductions (LMR)", &[]),
            MateDistance => ("Mate distance pruning", &[]),
            Razoring => ("Razoring", &[]),
            Rfp => ("Reverse futility pruning (RFP)", &[]),
            Lmp => ("Late move pruning (LMP)", &[]),
            Futility => ("Futility pruning", &[]),
            NullMove => ("Null-move pruning", &[]),
            MainSee => ("Main SEE pruning", &[]),
            Delta => ("Delta pruning", &[]),
            QSee => ("Quiescence SEE pruning", &[]),
            TerminalNodes => ("Terminal nodes", &[]),
            Draws => ("Draw returns", &[]),
            Tt => ("Transposition tables", &[]),
        }
    }

    pub fn mask(self) -> u64 {
        let (_, children) = self.tree();
        if children.is_empty() {
            1 << (self as u8)
        } else {
            children.iter().fold(0, |bits, child| bits | child.mask())
        }
    }
}

/// Masks refer to report leaves. Group checkboxes operate on all their leaves.
#[derive(Clone, Copy, Debug)]
pub struct ReportSettings {
    pub printed: u64,
    pub highlighted: u64,
}

impl Default for ReportSettings {
    fn default() -> Self {
        Self {
            printed: u64::MAX,
            highlighted: 0,
        }
    }
}

impl ReportSettings {
    pub(super) fn print(self, section: ReportSection, body: impl FnOnce()) {
        let mask = section.mask();
        if self.printed & mask == 0 {
            return;
        }

        if self.highlighted & mask == mask && io::stdout().is_terminal() {
            // Keep other search threads' output outside the colored block.
            let stream = StandardStream::stdout(ColorChoice::Always);
            let mut output = stream.lock();
            let _ = output.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            body();
            let _ = output.reset();
        } else {
            body();
        }
    }
}
