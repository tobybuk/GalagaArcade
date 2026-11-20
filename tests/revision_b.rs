use galaga_reconstructed::{formation::Formation, Game, State};

#[test]
fn reserve_icons_use_rev_b_eight_slots_and_preserve_stage_token_tiles() {
    let mut game = Game::default();
    game.tile_ram[..0x40].fill(0x24);
    game.reserve_fighters = 7;
    game.tile_ram[0x1b] = 0x36;
    game.tile_ram[0x19] = 0x49;
    game.draw_resv_ships();
    assert_eq!(game.tile_ram[0x1d], 0x4a);
    assert_eq!(game.tile_ram[0x1b], 0x36);
    assert_eq!(game.tile_ram[0x19], 0x49);
    assert_eq!(game.tile_ram[0x11], 0x4a);
    assert_eq!(game.tile_ram[0x0f], 0x24);
    game.reserve_fighters = 8;
    game.draw_resv_ships();
    assert_eq!(game.tile_ram[0x0f], 0x4a);
}

#[test]
fn rev_b_bomb_updater_releases_invalid_objects() {
    let mut game = Game::default();
    game.objects[52].state = State::Bomb;
    game.objects[52].x = 50;
    game.objects[52].code = 0x31;
    game.objects[53].state = State::Bomb;
    game.objects[53].x = 0;
    game.f_1ea4();
    assert_eq!(game.objects[52].state, State::Inactive);
    assert_eq!(game.objects[53].state, State::Inactive);
}

#[test]
fn formation_coordinate_carry_retains_original_ninth_bit() {
    let mut formation = Formation::default();
    formation.pulsing = true;
    formation.direction = 0;
    formation.coordinates[0] = 0;
    formation.pulse(0);
    assert_eq!(formation.coordinates[0], 0x1ff);
}
