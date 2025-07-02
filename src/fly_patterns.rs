use rand::Rng;

pub struct EnemyPatterns;

impl EnemyPatterns {
    pub fn get_initial_pattern(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        vec![
            ("b2_1", "b-2.png", board_width * 0.2, board_height * 0.1),
            ("b2_2", "b-2.png", board_width * 0.4, board_height * 0.1),
            ("b2_3", "b-2.png", board_width * 0.6, board_height * 0.1),
            ("b2_4", "b-2.png", board_width * 0.8, board_height * 0.1),
            ("tiki_1", "tiki_fly.png", board_width * 0.15, board_height * 0.2),
            ("tiki_2", "tiki_fly.png", board_width * 0.3, board_height * 0.2),
            ("tiki_3", "tiki_fly.png", board_width * 0.5, board_height * 0.2),
            ("tiki_4", "tiki_fly.png", board_width * 0.7, board_height * 0.2),
            ("tiki_5", "tiki_fly.png", board_width * 0.85, board_height * 0.2),
            ("northrop_1", "northrop.png", board_width * 0.25, board_height * 0.3),
            ("northrop_2", "northrop.png", board_width * 0.4, board_height * 0.3),
            ("northrop_3", "northrop.png", board_width * 0.6, board_height * 0.3),
            ("northrop_4", "northrop.png", board_width * 0.75, board_height * 0.3),
        ]
    }

    pub fn get_pattern_1(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // Diamond formation
        vec![
            ("b2_1", "b-2.png", board_width * 0.5, board_height * 0.05),
            ("b2_2", "b-2.png", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b-2.png", board_width * 0.7, board_height * 0.15),
            ("tiki_1", "tiki_fly.png", board_width * 0.1, board_height * 0.25),
            ("tiki_2", "tiki_fly.png", board_width * 0.5, board_height * 0.25),
            ("tiki_3", "tiki_fly.png", board_width * 0.9, board_height * 0.25),
            ("northrop_1", "northrop.png", board_width * 0.2, board_height * 0.35),
            ("northrop_2", "northrop.png", board_width * 0.4, board_height * 0.35),
            ("northrop_3", "northrop.png", board_width * 0.6, board_height * 0.35),
            ("northrop_4", "northrop.png", board_width * 0.8, board_height * 0.35),
        ]
    }

    pub fn get_pattern_2(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // V formation
        vec![
            ("b2_1", "b-2.png", board_width * 0.1, board_height * 0.1),
            ("b2_2", "b-2.png", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b-2.png", board_width * 0.5, board_height * 0.2),
            ("b2_4", "b-2.png", board_width * 0.7, board_height * 0.15),
            ("b2_5", "b-2.png", board_width * 0.9, board_height * 0.1),
            ("tiki_1", "tiki_fly.png", board_width * 0.2, board_height * 0.3),
            ("tiki_2", "tiki_fly.png", board_width * 0.4, board_height * 0.25),
            ("tiki_3", "tiki_fly.png", board_width * 0.6, board_height * 0.25),
            ("tiki_4", "tiki_fly.png", board_width * 0.8, board_height * 0.3),
            ("northrop_1", "northrop.png", board_width * 0.35, board_height * 0.4),
            ("northrop_2", "northrop.png", board_width * 0.65, board_height * 0.4),
        ]
    }

    pub fn get_pattern_3(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // Circular formation
        let center_x = board_width * 0.5;
        let center_y = board_height * 0.25;
        let radius = board_width * 0.2;

        vec![
            ("b2_1", "b-2.png", center_x + radius * 0.0, center_y - radius),
            ("b2_2", "b-2.png", center_x + radius * 0.707, center_y - radius * 0.707),
            ("b2_3", "b-2.png", center_x + radius, center_y),
            ("b2_4", "b-2.png", center_x + radius * 0.707, center_y + radius * 0.707),
            ("b2_5", "b-2.png", center_x, center_y + radius),
            ("b2_6", "b-2.png", center_x - radius * 0.707, center_y + radius * 0.707),
            ("b2_7", "b-2.png", center_x - radius, center_y),
            ("b2_8", "b-2.png", center_x - radius * 0.707, center_y - radius * 0.707),
            ("tiki_1", "tiki_fly.png", center_x, center_y),
            ("tiki_2", "tiki_fly.png", center_x + radius * 0.5, center_y),
            ("tiki_3", "tiki_fly.png", center_x - radius * 0.5, center_y),
            ("northrop_1", "northrop.png", board_width * 0.1, board_height * 0.4),
            ("northrop_2", "northrop.png", board_width * 0.9, board_height * 0.4),
        ]
    }

    pub fn get_pattern_4(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        let mut enemies = Vec::new();
        let mut rng = rand::thread_rng();

        for i in 1..=6 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.05..0.2) * board_height;
            let id: &'static str = Box::leak(format!("b2_{}", i).into_boxed_str());
            enemies.push((id, "b-2.png", x, y));
        }

        for i in 1..=8 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.2..0.35) * board_height;
            let id: &'static str = Box::leak(format!("tiki_{}", i).into_boxed_str());
            enemies.push((id, "tiki_fly.png", x, y));
        }

        for i in 1..=5 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.35..0.45) * board_height;
            let id: &'static str = Box::leak(format!("northrop_{}", i).into_boxed_str());
            enemies.push((id, "northrop.png", x, y));
        }

        enemies
    }
}