use crate::world::{World};
use crate::game::{Strategy};
use std::collections::VecDeque;
use macroquad::prelude::*;

/// Colour for a cell; cells that just switched strategy are drawn paler
fn cell_color(strategy: Strategy, switched: bool) -> Color {
    let base = match strategy {
        Strategy::Cooperate => Color::from_rgba(40, 230, 110, 255),
        Strategy::Defect => Color::from_rgba(220, 50, 50, 255),
        Strategy::Loner => Color::from_rgba(240, 190, 30, 255),
    };
    if switched {
        Color::new((base.r + 1.0) / 2.0, (base.g + 1.0) / 2.0, (base.b + 1.0) / 2.0, 1.0)
    } else {
        base
    }
}

pub struct App {
    pub w: usize,
    pub h: usize,
    pub hcap: usize,
    pub csize: f32,
    pub world: World,
    pub history: VecDeque<[usize; 3]>,
    pub paused: bool,
    pub steps_per_second: f32,
    pub accumulator: f32,
    pub brush: Strategy,
}

impl App {
    pub fn new(w: usize, h: usize, hist: usize, csize: f32) -> Self {
        Self {
            w: w,
            h: h,
            hcap: hist,
            csize: csize,
            world: World::new(w, h, 2026),
            history: VecDeque::with_capacity(hist),
            paused: false,
            steps_per_second: 15.0,
            accumulator: 0.0,
            brush: Strategy::Defect,
        }
    }

    pub fn step(&mut self) {
        self.world.step();
        if self.history.len() == self.hcap {
            self.history.pop_front();
        }
        self.history.push_back(self.world.census());
    }

    pub fn handle_input(&mut self, cell: f32) {
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        if is_key_pressed(KeyCode::N) {
            self.step(); // single step, handy while paused
        }
        if is_key_pressed(KeyCode::R) {
            self.world.randomize();
            self.history.clear();
        }
        if is_key_pressed(KeyCode::L) {
            self.world.loners_enabled = !self.world.loners_enabled;
            self.world.randomize();
            self.history.clear();
        }
        if is_key_pressed(KeyCode::Up) {
            self.world.payoffs.temptation += 0.05;
        }
        if is_key_pressed(KeyCode::Down) {
            self.world.payoffs.temptation = (self.world.payoffs.temptation - 0.05).max(1.0);
        }
        if is_key_pressed(KeyCode::Right) {
            self.steps_per_second = (self.steps_per_second * 1.5).min(240.0);
        }
        if is_key_pressed(KeyCode::Left) {
            self.steps_per_second = (self.steps_per_second / 1.5).max(1.0);
        }
        for (key, strategy) in [
            (KeyCode::Key1, Strategy::Cooperate),
            (KeyCode::Key2, Strategy::Defect),
            (KeyCode::Key3, Strategy::Loner),
        ] {
            if is_key_pressed(key) {
                self.brush = strategy;
            }
        }

        // Paint a 3x3 block of cells under the mouse
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (gx, gy) = ((mx / cell) as isize, (my / cell) as isize);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (x, y) = (gx + dx, gy + dy);
                    if (0..self.w as isize).contains(&x) && (0..self.h as isize).contains(&y) {
                        self.world.set(x as usize, y as usize, self.brush);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        self.accumulator += dt;
        let interval = 1.0 / self.steps_per_second;
        // Limit catch-up work so one slow frame can't snowball
        let mut budget = 8;
        while self.accumulator >= interval && budget > 0 {
            self.step();
            self.accumulator -= interval;
            budget -= 1;
        }
        if budget == 0 {
            self.accumulator = 0.0;
        }
    }

    pub fn draw_grid(&self, cell: f32) {
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.world.get(x, y);
                let color = cell_color(c.strategy, c.strategy != c.previous);
                draw_rectangle(x as f32 * cell, y as f32 * cell, cell, cell, color);
            }
        }
    }

    pub fn draw_chart(&self, top: f32, width: f32) {
        draw_rectangle(0.0, top, width, self.csize, Color::from_rgba(18, 18, 24, 255));
        let total = (self.w * self.h) as f32;
        let dx = width / self.hcap as f32;
        let plot_h = self.csize - 40.0;
        let base = top + self.csize - 10.0;

        for strategy in Strategy::ALL {
            let color = cell_color(strategy, false);
            let points: Vec<(f32, f32)> = self
                .history
                .iter()
                .enumerate()
                .map(|(i, counts)| {
                    let frac = counts[strategy.index()] as f32 / total;
                    (i as f32 * dx, base - frac * plot_h)
                })
                .collect();
            for pair in points.windows(2) {
                let ((x1, y1), (x2, y2)) = (pair[0], pair[1]);
                draw_line(x1, y1, x2, y2, 2.0, color);
            }
        }

        let status = format!(
            "gen {}  b={:.2}  speed {:.0}/s  brush: {}  {}",
            self.world.generation,
            self.world.payoffs.temptation,
            self.steps_per_second,
            self.brush,
            if self.paused { "[PAUSED]" } else { "" },
        );
        draw_text(&status, 8.0, top + 20.0, 20.0, WHITE);
    }
}
