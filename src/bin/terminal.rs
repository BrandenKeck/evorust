//! Animate the simulation in the terminal with ANSI colours.
//! Run with: cargo run --release --bin terminal -- [generations]

use evo_sim::{Strategy, World};
use std::fmt::Write as _;
use std::{env, thread, time::Duration};

fn color(s: Strategy) -> &'static str {
    match s {
        Strategy::Cooperate => "\x1b[44m", // blue background
        Strategy::Defect => "\x1b[41m",    // red background
        Strategy::Loner => "\x1b[43m",     // yellow background
    }
}

fn render(world: &World) -> String {
    let mut out = String::from("\x1b[H"); // cursor to top-left
    for y in 0..world.height() {
        for x in 0..world.width() {
            out.push_str(color(world.get(x, y).strategy));
            out.push_str("  "); // two spaces = one roughly square cell
        }
        out.push_str("\x1b[0m\n"); // reset colour at the end of each row
    }
    let [c, d, l] = world.census();
    let _ = writeln!(out, "gen {:>5}  C {c:>5}  D {d:>5}  L {l:>5}", world.generation);
    out
}

fn main() {
    let generations: u64 = env::args()
        .nth(1)
        .map(|s| s.parse().expect("generations must be a whole number"))
        .unwrap_or(500);

    let mut world = World::new(40, 25, 2026);
    print!("\x1b[2J"); // clear the screen once
    for _ in 0..generations {
        print!("{}", render(&world));
        world.step();
        thread::sleep(Duration::from_millis(60));
    }
}