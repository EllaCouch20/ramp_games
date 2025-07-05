use rand::Rng;
use image::{load_from_memory, RgbaImage};
use pelican_ui::drawable::Image;
use pelican_ui::Context;

pub struct EnemyPatterns;

impl EnemyPatterns {
    pub fn get_initial_pattern(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        vec![
            ("b2_1", "b2", board_width * 0.2, board_height * 0.1),
            ("b2_2", "b2", board_width * 0.4, board_height * 0.1),
            ("b2_3", "b2", board_width * 0.6, board_height * 0.1),
            ("b2_4", "b2", board_width * 0.8, board_height * 0.1),
            ("tiki_1", "tiki_fly", board_width * 0.15, board_height * 0.2),
            ("tiki_2", "tiki_fly", board_width * 0.3, board_height * 0.2),
            ("tiki_3", "tiki_fly", board_width * 0.5, board_height * 0.2),
            ("tiki_4", "tiki_fly", board_width * 0.7, board_height * 0.2),
            ("tiki_5", "tiki_fly", board_width * 0.85, board_height * 0.2),
            ("northrop_1", "northrop", board_width * 0.25, board_height * 0.3),
            ("northrop_2", "northrop", board_width * 0.4, board_height * 0.3),
            ("northrop_3", "northrop", board_width * 0.6, board_height * 0.3),
            ("northrop_4", "northrop", board_width * 0.75, board_height * 0.3),
        ]
    }

    pub fn get_pattern_1(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        vec![
            ("b2_1", "b2", board_width * 0.5, board_height * 0.05),
            ("b2_2", "b2", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b2", board_width * 0.7, board_height * 0.15),
            ("tiki_1", "tiki_fly", board_width * 0.1, board_height * 0.25),
            ("tiki_2", "tiki_fly", board_width * 0.5, board_height * 0.25),
            ("tiki_3", "tiki_fly", board_width * 0.9, board_height * 0.25),
            ("northrop_1", "northrop", board_width * 0.2, board_height * 0.35),
            ("northrop_2", "northrop", board_width * 0.4, board_height * 0.35),
            ("northrop_3", "northrop", board_width * 0.6, board_height * 0.35),
            ("northrop_4", "northrop", board_width * 0.8, board_height * 0.35),
        ]
    }

    pub fn get_pattern_2(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        vec![
            ("b2_1", "b2", board_width * 0.1, board_height * 0.1),
            ("b2_2", "b2", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b2", board_width * 0.5, board_height * 0.2),
            ("b2_4", "b2", board_width * 0.7, board_height * 0.15),
            ("b2_5", "b2", board_width * 0.9, board_height * 0.1),
            ("tiki_1", "tiki_fly", board_width * 0.2, board_height * 0.3),
            ("tiki_2", "tiki_fly", board_width * 0.4, board_height * 0.25),
            ("tiki_3", "tiki_fly", board_width * 0.6, board_height * 0.25),
            ("tiki_4", "tiki_fly", board_width * 0.8, board_height * 0.3),
            ("northrop_1", "northrop", board_width * 0.35, board_height * 0.4),
            ("northrop_2", "northrop", board_width * 0.65, board_height * 0.4),
        ]
    }

    pub fn get_pattern_3(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        let center_x = board_width * 0.5;
        let center_y = board_height * 0.25;
        let radius = board_width * 0.2;

        vec![
            ("b2_1", "b2", center_x + radius * 0.0, center_y - radius),
            ("b2_2", "b2", center_x + radius * 0.707, center_y - radius * 0.707),
            ("b2_3", "b2", center_x + radius, center_y),
            ("b2_4", "b2", center_x + radius * 0.707, center_y + radius * 0.707),
            ("b2_5", "b2", center_x, center_y + radius),
            ("b2_6", "b2", center_x - radius * 0.707, center_y + radius * 0.707),
            ("b2_7", "b2", center_x - radius, center_y),
            ("b2_8", "b2", center_x - radius * 0.707, center_y - radius * 0.707),
            ("tiki_1", "tiki_fly", center_x, center_y),
            ("tiki_2", "tiki_fly", center_x + radius * 0.5, center_y),
            ("tiki_3", "tiki_fly", center_x - radius * 0.5, center_y),
            ("northrop_1", "northrop", board_width * 0.1, board_height * 0.4),
            ("northrop_2", "northrop", board_width * 0.9, board_height * 0.4),
        ]
    }

    pub fn get_random_pattern(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        let mut rng = rand::thread_rng();
        let pattern_choice = rng.gen_range(0..4);

        match pattern_choice {
            0 => Self::get_initial_pattern(board_width, board_height),
            1 => Self::get_pattern_1(board_width, board_height),
            2 => Self::get_pattern_2(board_width, board_height),
            _ => Self::get_pattern_3(board_width, board_height),
        }
    }
}
