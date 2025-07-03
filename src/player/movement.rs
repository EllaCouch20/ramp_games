use pelican_ui::Context;
use pelican_game_engine::Gameboard;

use super::{PlayerState, STEP};

pub fn handle_movement_by_state(ctx: &mut Context, board: &mut Gameboard, player_state: PlayerState) {
    let (maxw, _) = board.0.size(ctx);

    if let Some(sprite) = board.2.get_mut("player") {
        let current_pos = sprite.position(ctx).0;

        match player_state {
            PlayerState::MovingLeft { speed: _, .. } => {
                if current_pos > 5.0 {
                    sprite.adjustments().0 -= STEP;
                }
            }

            PlayerState::MovingRight { speed: _, .. } => {
                if current_pos < maxw - sprite.dimensions().0 - 5.0 {
                    sprite.adjustments().0 += STEP;
                }
            }

            PlayerState::MovingBoth { left_speed: _, right_speed: _, .. } => {
                // Handle both directions movement if needed
                // Currently no-op as in original code
            }

            PlayerState::Idle { .. } | PlayerState::Shooting { .. } | PlayerState::Destroyed => {
                // No movement for these states
            }
        }
    }
}