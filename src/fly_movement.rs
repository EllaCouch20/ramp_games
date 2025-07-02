use pelican_ui::Context;
use pelican_game_engine::Gameboard;
use std::collections::HashMap;

const PULSE_AMPLITUDE: f32 = 5.0;
const PULSE_SPEED: f32 = 0.1;

pub struct EnemyMovement;

impl EnemyMovement {
    pub fn update_enemy_pulse(
        ctx: &mut Context, 
        board: &mut Gameboard,
        pulse_time: &mut f32,
        base_positions: &HashMap<String, (f32, f32)>
    ) {
        *pulse_time += PULSE_SPEED;

        let pulse_scale = (pulse_time.sin() + 1.0) * 0.5;
        let pulse_offset = pulse_scale * PULSE_AMPLITUDE;

        if base_positions.is_empty() {
            return;
        }

        let center_x = base_positions.values().map(|(x, _)| *x).sum::<f32>() / base_positions.len() as f32;
        let center_y = base_positions.values().map(|(_, y)| *y).sum::<f32>() / base_positions.len() as f32;

        for (enemy_id, &(base_x, base_y)) in base_positions.iter() {
            if let Some(sprite) = board.2.get_mut(enemy_id) {
                let dx = base_x - center_x;
                let dy = base_y - center_y;

                let distance = (dx * dx + dy * dy).sqrt();
                let (norm_dx, norm_dy) = if distance > 0.0 {
                    (dx / distance, dy / distance)
                } else {
                    (0.0, 0.0)
                };

                let pulse_x = norm_dx * pulse_offset;
                let pulse_y = norm_dy * pulse_offset;

                sprite.adjustments().0 = pulse_x;
                sprite.adjustments().1 = pulse_y;
            }
        }
    }
}