pub fn is_enemy(sprite_id: &str) -> bool {
    (sprite_id.starts_with("b2_") || 
     sprite_id.starts_with("tiki_") || 
     sprite_id.starts_with("northrop_")) &&
    !sprite_id.starts_with("player") &&
    !sprite_id.starts_with("bullet_") &&
    !sprite_id.starts_with("enemy_bullet_") &&
    !sprite_id.starts_with("explosion_")
}

pub fn is_enemy_bullet(sprite_id: &str) -> bool {
    sprite_id.starts_with("enemy_bullet_")
}

pub fn is_tiki(sprite_id: &str) -> bool {
    sprite_id.starts_with("tiki_")
}

pub fn count_active_enemies(board: &pelican_game_engine::Gameboard) -> usize {
    let count = board.2.keys().filter(|id| is_enemy(id)).count();
    println!("Active enemies: {}", count);
    count
}