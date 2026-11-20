use galaga_reconstructed::{Game, Object, State, FIGHTER};

#[test]
fn spin_uses_original_orientation_cycle_and_upright_stop() {
    let mut game = Game::default();
    game.objects[FIGHTER].code = 6;
    game.objects[FIGHTER].flags = 0;
    game.combat.capture_status[3] = 1;
    for _ in 0..24 {
        let (_, b) = game.c_2188_ship_spin(FIGHTER);
        assert_eq!(b, 0);
        assert!(game.objects[FIGHTER].code <= 6);
        assert!(game.objects[FIGHTER].flags <= 3);
    }
    assert_eq!((game.objects[FIGHTER].code, game.objects[FIGHTER].flags), (6, 0));
    game.combat.capture_status[3] = 0;
    assert_eq!(game.c_2188_ship_spin(FIGHTER), (0, 1));
}

#[test]
fn rev_b_capture_delays_transfer_until_text_expires_and_copies_controls() {
    let mut game = Game::default();
    game.combat.capture_boss_id = 0x30;
    game.objects[24] = Object { state: State::Homing, x: 128, y: 200, ..Object::default() };
    game.objects[FIGHTER].flags = 3;
    let fighter_x = game.objects[FIGHTER].x;
    game.combat.capture_status[4] = 1;
    game.f_19b2();
    assert_eq!(game.objects[FIGHTER].x, fighter_x);
    assert_eq!(game.objects[FIGHTER].colour, 7);
    assert_eq!(game.objects[0].x, 0);
    assert_eq!(game.game_timers[1], 6);
    game.game_timers[1] = 4;
    game.f_19b2();
    assert_eq!(game.objects[FIGHTER].x, fighter_x);
    assert_eq!(game.combat.sound_requests[9], 3);
    assert_eq!(game.motions[0].duration, 4);
    game.game_timers[1] = 0;
    game.f_19b2();
    assert_eq!(game.objects[FIGHTER].x, 0);
    assert_eq!((game.objects[0].code, game.objects[0].colour, game.objects[0].flags), (7, 7, 3));
    assert_eq!((game.objects[0].x, game.objects[0].y), (128, 216));
    game.objects[24].state = State::HomeReturn;
    for _ in 0..36 { game.f_19b2(); }
    assert_eq!(game.objects[0].y, 180);
    game.f_19b2();
    assert_eq!(game.objects[0].state, State::Standby);
    assert_eq!(game.objects[0].code, 6);
    assert_eq!(game.restart_stage, 2);
    assert_eq!(game.combat.capture_boss_id, 1);
}

#[test]
fn rescued_ship_lands_and_joins_at_original_positions() {
    let mut game = Game::default();
    game.objects[0] = Object { state: State::Rescued, x: 128, y: 288, code: 6, colour: 9, ..Object::default() };
    game.objects[FIGHTER].x = 113;
    game.combat.capture_boss_id = 0;
    game.combat.capture_status[1] = 2;
    game.main_task_enable[0x1d] = 1;
    game.main_task_enable[20] = 0; game.main_task_enable[21] = 0;
    for _ in 0..10 { game.f_2000(); }
    assert_eq!(game.two_fighters, 1);
    assert_eq!(game.objects[0].state, State::Inactive);
    assert_eq!(game.objects[FIGHTER - 1].x, 128);
    assert_eq!(game.objects[FIGHTER - 1].y, 297);
    assert_eq!(game.objects[FIGHTER].x, 113);
    assert_eq!(game.main_task_enable[0x1d], 0);
    assert_eq!(game.main_task_enable[20], 1);
    assert_eq!(game.main_task_enable[21], 1);
    game.fighter_control(8);
    assert_eq!(game.objects[FIGHTER - 1].x, game.objects[FIGHTER].x + 15);
}

#[test]
fn beam_capture_connects_then_transitions_to_original_captured_task() {
    let mut game = Game::default();
    game.main_task_enable[0x18] = 1;
    game.combat.capture_boss_id = 0x30;
    game.objects[24] = Object { state: State::Homing, x: 129, y: 209, ..Object::default() };
    game.objects[FIGHTER].x = 129;
    game.capture_pause = 12;
    game.combat.capture_status = [129, 0, 1, 1, 0];
    for frame in 0..1500 {
        game.counters[0] = frame as u8;
        if game.main_task_enable[0x18] != 0 { game.f_2222(); }
        if game.main_task_enable[0x1c] != 0 { game.f_20f2(); }
        if game.main_task_enable[0x11] != 0 { break; }
    }
    assert_eq!(game.capture_connected, 1);
    assert_eq!(game.main_task_enable[0x18], 0);
    assert_eq!(game.main_task_enable[0x1c], 0);
    assert_eq!(game.main_task_enable[0x11], 1);
    assert_eq!(game.motions[0].path, 0x046b);
    assert_eq!(game.combat.capture_status[4], 1);
    assert_eq!(game.objects[FIGHTER].colour, 7);
    assert!(game.tile_ram[0x40..0x3c0].iter().all(|&b| b == 0x24));
}
