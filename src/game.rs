//! The game itself: which strategies exist and what they earn.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    Cooperate,
    Defect,
    Loner,
}

impl Strategy {
    pub const ALL: [Strategy; 3] = [Strategy::Cooperate, Strategy::Defect, Strategy::Loner];

    /// Position in ALL (0, 1 or 2), handy for indexing arrays
    pub fn index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Strategy::Cooperate => "Cooperator",
            Strategy::Defect => "Defector",
            Strategy::Loner => "Loner",
        };
        write!(f, "{name}")
    }
}

/// temptation (b): what a defector earns by exploiting a cooperator
/// loner (σ): the small safe payoff when anyone involved is a loner
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payoffs {
    pub temptation: f32,
    pub loner: f32,
}

impl Default for Payoffs {
    fn default() -> Self {
        Self { temptation: 2.6, loner: 0.1 }
    }
}

impl Payoffs {
    pub fn score(&self, me: Strategy, them: Strategy) -> f32 {
        use Strategy::*;
        match (me, them) {
            (Loner, _) | (_, Loner) => self.loner,
            (Cooperate, Cooperate) => 1.0,
            (Defect, Cooperate) => self.temptation,
            (Cooperate, Defect) | (Defect, Defect) => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defection_is_tempting() {
        let p = Payoffs::default();
        assert!(p.score(Strategy::Defect, Strategy::Cooperate) > p.score(Strategy::Cooperate, Strategy::Cooperate));
        assert_eq!(p.score(Strategy::Cooperate, Strategy::Defect), 0.0);
    }

    #[test]
    fn loners_always_get_sigma() {
        let p = Payoffs::default();
        for s in Strategy::ALL {
            assert_eq!(p.score(Strategy::Loner, s), p.loner);
            assert_eq!(p.score(s, Strategy::Loner), p.loner);
        }
    }
}