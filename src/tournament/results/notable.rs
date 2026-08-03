use crate::tournament::GameRecord;

#[derive(Clone, Debug, PartialEq)]
pub enum NotableReason {
    LargeEvalSwing {
        from_eval: i32,
        to_eval: i32,
        swing: i32,
        ply: usize,
    },
    MultipleEvalSwings {
        count: usize,
        moves: Vec<(i32, usize)>, // Stored as (Swing, ply)
    },
    Comeback {
        average_eval: i32,
    },
    AbnormalEval, // this can include weird average eval, notable jumps back and forth
    Upset,
    LongGame,
    ShortGame,
}

#[derive(Clone)]
pub struct NotableGame {
    pub game_record: GameRecord,
    pub reasons: Vec<NotableReason>,
    pub importance: i32,
}

impl NotableGame {
    pub fn print_summary(&self, index: usize) {
        if self.game_record.opening_name.is_some() {
            println!(
                "Game: {}. Opening: {}. Importance: {} | Result: {:?} | Reasons:",
                index,
                self.game_record.opening_name.as_ref().unwrap(),
                self.importance,
                self.game_record.result
            );
        } else {
            println!(
                "Game: {}. Importance: {} | Result: {:?} | Reasons:",
                index, self.importance, self.game_record.result
            );
        }

        for reason in &self.reasons {
            match reason {
                NotableReason::LargeEvalSwing {
                    from_eval,
                    to_eval,
                    swing,
                    ply,
                } => {
                    println!(
                        "     - Large swing: {:+} -> {:+} ({:+} cp) at ply {}",
                        from_eval, to_eval, swing, ply
                    );
                }

                NotableReason::MultipleEvalSwings { count, moves } => {
                    print!("     - Multiple swings: {} [", count);

                    for (i, (swing, ply)) in moves.iter().enumerate() {
                        if i > 0 {
                            print!(", ");
                        }

                        print!("{:+} cp @ {}", swing, ply);
                    }

                    println!("]");
                }

                NotableReason::Comeback { average_eval } => {
                    println!("     - Comeback: average eval was {:+} cp", average_eval);
                }

                NotableReason::AbnormalEval => {
                    println!("     - Abnormal evaluation behavior");
                }

                NotableReason::Upset => {
                    println!("     - Upset");
                }

                NotableReason::LongGame => {
                    println!("     - Long game");
                }

                NotableReason::ShortGame => {
                    println!("     - Short game");
                }
            }
        }
    }
}
