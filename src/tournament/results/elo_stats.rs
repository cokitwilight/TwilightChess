use crate::tournament::TournamentResult;

impl TournamentResult {
    pub fn elo_stats(&self) -> Option<EloStats> {
        calculate_elo_stats(
            self.engine_1_wins,
            self.engine_1_draws,
            self.engine_1_losses,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EloStats {
    /// Engine 1's average score: win = 1.0, draw = 0.5, loss = 0.0.
    pub score_rate: f64,

    /// Engine 1 estimated Elo relative to Engine 2.
    /// Positive means Engine 1 is stronger.
    pub elo_difference: f64,

    /// Approximate 95% confidence interval.
    pub elo_ci_low: f64,
    pub elo_ci_high: f64,

    /// Standard error of the mean game score.
    pub score_standard_error: f64,

    /// Approximate 95% CI in score space.
    pub score_ci_low: f64,
    pub score_ci_high: f64,

    /// Probability Engine 1 is actually stronger than Engine 2.
    /// 0.95 = 95% LOS.
    pub likelihood_of_superiority: f64,

    pub draw_rate: f64,
    pub decisive_rate: f64,

    pub games: usize,
}

pub fn calculate_elo_stats(wins: usize, draws: usize, losses: usize) -> Option<EloStats> {
    let games = wins + draws + losses;

    if games == 0 {
        return None;
    }

    let n = games as f64;
    let w = wins as f64;
    let d = draws as f64;
    let l = losses as f64;

    // Individual game scores:
    // win  = 1.0
    // draw = 0.5
    // loss = 0.0
    let score_rate = (w + 0.5 * d) / n;

    // E[X^2]:
    // win:  1^2   = 1
    // draw: 0.5^2 = 0.25
    // loss: 0^2   = 0
    let mean_square = (w + 0.25 * d) / n;

    // Population-style estimate of variance.
    //
    // Then apply Bessel correction for sample variance.
    let mut variance = mean_square - score_rate * score_rate;

    if games > 1 {
        variance *= n / (n - 1.0);
    }

    variance = variance.max(0.0);

    let standard_error = (variance / n).sqrt();

    // 95% normal confidence interval.
    const Z_95: f64 = 1.959963984540054;

    let score_ci_low = (score_rate - Z_95 * standard_error).clamp(0.0, 1.0);

    let score_ci_high = (score_rate + Z_95 * standard_error).clamp(0.0, 1.0);

    let elo_difference = score_to_elo(score_rate);
    let elo_ci_low = score_to_elo(score_ci_low);
    let elo_ci_high = score_to_elo(score_ci_high);

    // H0: score_rate = 0.5
    //
    // Convert distance from 50% into a normal z-score.
    let likelihood_of_superiority = if standard_error > 0.0 {
        let z = (score_rate - 0.5) / standard_error;
        normal_cdf(z)
    } else if score_rate > 0.5 {
        1.0
    } else if score_rate < 0.5 {
        0.0
    } else {
        0.5
    };

    Some(EloStats {
        score_rate,
        elo_difference,
        elo_ci_low,
        elo_ci_high,

        score_standard_error: standard_error,
        score_ci_low,
        score_ci_high,

        likelihood_of_superiority,

        draw_rate: d / n,
        decisive_rate: (w + l) / n,

        games,
    })
}

fn score_to_elo(score: f64) -> f64 {
    // Avoid +/- infinity for undefeated tests.
    const EPSILON: f64 = 1e-10;

    let score = score.clamp(EPSILON, 1.0 - EPSILON);

    400.0 * (score / (1.0 - score)).log10()
}

fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

// Approximation of the error function.
//
// More than accurate enough for tournament statistics / LOS.
fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation.
    const P: f64 = 0.3275911;
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + P * x);

    let y = 1.0 - (((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t) * (-x * x).exp();

    sign * y
}
