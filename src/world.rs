//! The grid world: every cell plays its neighbours, then copies the most
//! successful strategy nearby (with occasional random mutation).

use crate::game::{Payoffs, Strategy};
use crate::rng::XorShift64;

/// The 8 surrounding cells plus the cell itself
const OFFSETS: [(isize, isize); 9] = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0),  (0, 0),  (1, 0),
    (-1, 1),  (0, 1),  (1, 1),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub strategy: Strategy,
    /// Last generation's strategy, so the display can highlight cells that switched
    pub previous: Strategy,
}

pub struct World {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    next: Vec<Cell>,
    scores: Vec<f32>,
    rng: XorShift64,
    pub payoffs: Payoffs,
    pub mutation_rate: f32,
    pub loners_enabled: bool,
    pub generation: u64,
}

impl World {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        assert!(width > 0 && height > 0, "world must not be empty");
        let n = width * height;
        let blank = Cell { strategy: Strategy::Cooperate, previous: Strategy::Cooperate };
        let mut world = Self {
            width,
            height,
            cells: vec![blank; n],
            next: vec![blank; n],
            scores: vec![0.0; n],
            rng: XorShift64::new(seed),
            payoffs: Payoffs::default(),
            mutation_rate: 0.001,
            loners_enabled: true,
            generation: 0,
        };
        world.randomize();
        world
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Strategies currently allowed to exist
    fn allowed(&self) -> &'static [Strategy] {
        if self.loners_enabled {
            &Strategy::ALL
        } else {
            &Strategy::ALL[..2]
        }
    }

    fn random_strategy(&mut self) -> Strategy {
        let allowed = self.allowed();
        allowed[self.rng.below(allowed.len())]
    }

    pub fn randomize(&mut self) {
        for i in 0..self.cells.len() {
            let s = self.random_strategy();
            self.cells[i] = Cell { strategy: s, previous: s };
        }
        self.generation = 0;
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.idx(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, strategy: Strategy) {
        let i = self.idx(x, y);
        self.cells[i].strategy = strategy;
    }

        /// Indexes of the 9 cells around (x, y), wrapping at the edges
    fn neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
        let (w, h) = (self.width as isize, self.height as isize);
        OFFSETS.iter().map(move |&(dx, dy)| {
            let nx = (x as isize + dx).rem_euclid(w) as usize;
            let ny = (y as isize + dy).rem_euclid(h) as usize;
            self.idx(nx, ny)
        })
    }

    pub fn step(&mut self) {
        // Phase 1: every cell plays its 9 neighbours (including itself)
        for y in 0..self.height {
            for x in 0..self.width {
                let i = self.idx(x, y);
                let me = self.cells[i].strategy;
                let total: f32 = self
                    .neighbors(x, y)
                    .map(|j| self.payoffs.score(me, self.cells[j].strategy))
                    .sum();
                self.scores[i] = total;
            }
        }

        // Phase 2: copy the best-scoring neighbour, sometimes mutate
        for y in 0..self.height {
            for x in 0..self.width {
                let i = self.idx(x, y);
                let best = self
                    .neighbors(x, y)
                    .max_by(|&a, &b| self.scores[a].total_cmp(&self.scores[b]))
                    .expect("a neighbourhood always has 9 cells");
                let mut strategy = self.cells[best].strategy;
                if self.rng.next_f32() < self.mutation_rate {
                    strategy = self.random_strategy();
                }
                self.next[i] = Cell { strategy, previous: self.cells[i].strategy };
            }
        }

        // Phase 3: the new generation becomes the current one
        std::mem::swap(&mut self.cells, &mut self.next);
        self.generation += 1;
    }

        /// How many cells use each strategy, indexed by Strategy::index()
    pub fn census(&self) -> [usize; 3] {
        let mut counts = [0; 3];
        for cell in &self.cells {
            counts[cell.strategy.index()] += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_size_is_conserved() {
        let mut w = World::new(20, 15, 42);
        for _ in 0..50 {
            w.step();
            assert_eq!(w.census().iter().sum::<usize>(), 20 * 15);
        }
        assert_eq!(w.generation, 50);
    }

    #[test]
    fn all_cooperators_stay_cooperators() {
        let mut w = World::new(10, 10, 1);
        w.mutation_rate = 0.0;
        for y in 0..10 {
            for x in 0..10 {
                w.set(x, y, Strategy::Cooperate);
            }
        }
        w.step();
        assert_eq!(w.census(), [100, 0, 0]);
    }

    #[test]
    fn neighbors_wrap_around() {
        let w = World::new(5, 5, 1);
        let n: Vec<usize> = w.neighbors(0, 0).collect();
        assert_eq!(n.len(), 9);
        assert!(n.contains(&w.idx(4, 4))); // top-left touches bottom-right
    }

    #[test]
    fn same_seed_same_history() {
        let mut a = World::new(30, 30, 7);
        let mut b = World::new(30, 30, 7);
        for _ in 0..20 {
            a.step();
            b.step();
        }
        assert_eq!(a.census(), b.census());
    }
}