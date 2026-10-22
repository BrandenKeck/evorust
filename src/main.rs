// Required modules
use evo_sim::{App};
use evo_sim::config::config::{get_config};
use macroquad::prelude::*;


fn window_conf() -> Conf {
    let config = get_config();
    Conf {
        window_title: "Evolution: Cooperate, Defect or Walk Away".to_owned(),
        window_width: 720,
        window_height: 720 + config.general.csize as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = get_config();
    let mut app = App::new(config.general.grid_w, config.general.grid_h, config.general.hist, config.general.csize);
    loop {
        let grid_px = screen_width().min(screen_height() - config.general.grid_h as f32);
        let cell = grid_px / config.general.grid_w as f32;

        app.handle_input(cell);
        app.update(get_frame_time());

        clear_background(BLACK);
        app.draw_grid(cell);
        app.draw_chart(grid_px, screen_width());

        next_frame().await
    }
}