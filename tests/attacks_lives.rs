//! Assembly-derived checks for routines independently byte-mapped to rev. B.
//! These are not a claim of an independent whole-game reference run.
use galaga_reconstructed::{executive::Background, motion::Motion, Game, Object, State, FIGHTER};

fn rest(game: &mut Game, id: usize, colour: u8) {
    game.objects[id / 2] = Object { state: State::Standby, code: 6, colour, x: 100, y: 100, ..Object::default() };
}

#[test]
fn boss_queues_two_escorts_and_uses_original_score_attributes() {
    let mut game = Game::default();
    for id in [0x30, 0x32, 0x34, 0x36] { rest(&mut game, id, 0); }
    for id in [0x4a, 0x52, 0x5a, 0x58, 0x50, 0x48] { rest(&mut game, id, 2); }
    game.bomber_control[..3].copy_from_slice(&[1, 4, 4]);
    game.f_1b65();
    let ids: Vec<_> = game.boss_sorties.iter().map(|s| s.object_id).collect();
    assert_eq!(ids, [0x30, 0x48, 0x50, 0xff]);
    assert_eq!(&game.combat.boss_score_attributes[..2], &[13, 0xba]);
    assert_eq!(game.objects[24].state, State::Standby); // queue first, launch on subsequent calls
    for (slot, id) in [0x30, 0x48, 0x50].into_iter().enumerate() {
        game.f_1b65();
        assert_eq!(game.motions[slot].object_id, id);
        assert_eq!(game.motions[slot].path, 0x0411);
        assert_eq!(game.objects[id as usize / 2].state, State::Homing);
    }
}

#[test]
fn capture_toggle_and_captured_fighter_follow_state_bytes() {
    let mut game = Game::default();
    rest(&mut game, 0x32, 0);
    rest(&mut game, 2, 7);
    // Deliberately not equal to 1: the old C wrongly tests motion_slot.
    game.objects[1].motion_slot = 9;
    game.boss_escort_toggle = 1;
    game.bomber_control[..3].copy_from_slice(&[1, 4, 4]);
    game.f_1b65();
    assert_eq!(game.boss_sorties[0].object_id, 0xb2);
    assert_eq!(game.boss_sorties[0].path, 0x454);
    assert_eq!(game.boss_sorties[1].object_id, 0x82);
    assert_eq!(game.combat.capture_boss_flag, 1);
    assert_eq!(game.combat.capture_boss_id, 0x32);
    assert_eq!(&game.combat.boss_score_attributes[2..4], &[1, 0xb5]);
}

#[test]
fn launch_limits_and_bonus_bee_exclusion_are_original() {
    let mut game = Game::default();
    rest(&mut game, 8, 3);
    rest(&mut game, 10, 3);
    game.combat.bonus_bee_id = 8;
    game.bomber_control[..3].copy_from_slice(&[4, 4, 1]);
    game.flying_count = game.stage_parameters[4];
    game.f_1b65();
    assert_eq!(game.bomber_control[2], 1);
    assert!(!game.motions.iter().any(Motion::active));
    game.flying_count = 0;
    game.f_1b65();
    assert_eq!(game.motions[0].object_id, 10);
    assert_eq!(game.motions[0].flags, 0x81);
    assert_eq!(game.combat.sound_requests[0x13], 8);
    assert_eq!(game.motions[0].x, 100 << 7);
    assert_eq!(game.motions[0].y, (353 - 100) << 7);
}

#[test]
fn dive_turning_point_keeps_zero_duration_until_original_y_test() {
    let mut game = Game::default();
    let mut object = Object { state: State::Homing, ..Object::default() };
    // 035B is FC,2E; the branch proceeds to motion without loading duration.
    let mut motion = Motion { path: 0x035b, duration: 1, y: 0x2e00, x: 0x4000,
                              flags: 1, bomb_counter: 9, ..Motion::default() };
    game.mctl_fltpn_dspchr(&mut motion, &mut object, 0).unwrap();
    assert_eq!(motion.path, 0x035d);
    assert_eq!(motion.flags & 0x20, 0);
    assert_eq!(motion.duration, 1); // proximity expires it on the next step
}

#[test]
fn bomb_spawn_restores_rev_b_control_and_uses_pointer_low_byte() {
    let mut game = Game::default();
    game.objects[FIGHTER].x = 100;
    let mut object = Object { state: State::Homing, ..Object::default() };
    // Zero displacement isolates the original bomb setup after sprite output.
    let mut motion = Motion { duration: 2, y: 0x7000, x: 0x3000, flags: 1,
                              bomb_counter: 1, bomb_flags: 1, ..Motion::default() };
    game.objects[52].flags = 0;
    game.mctl_fltpn_dspchr(&mut motion, &mut object, 0).unwrap();
    assert_eq!(game.objects[52].state, State::Bomb);
    assert_eq!(game.objects[52].flags, 1);
    assert_eq!((game.objects[52].x, game.objects[52].y), (96, 129));
    // Original HL = 0469, divisor 55h, quotient 13, scaled rate 4 -> RR 2.
    // Dropping the pointer low byte gives rate 1 instead.
    assert_eq!(game.bomb_rates[0], [2, 0]);
    assert_eq!(motion.bomb_counter, game.bomb_reload);
}

#[test]
fn bomb_notification_alias_and_fighter_explosion_use_actual_ram_banks() {
    let mut game = Game::default();
    game.main_task_enable[8] = 0;
    game.objects[FIGHTER].x = 100;
    game.objects[FIGHTER].y = 0x100;
    game.sprite_ram[FIGHTER].x = 102;
    game.objects[52] = Object { state: State::Bomb, colour: 11, x: 100, y: 0x100, ..Object::default() };
    game.shared_9260[8] = 0x80; // 9268: detector skips this bomb
    game.f_05ee();
    assert_eq!(game.restart_stage, 0);
    game.shared_9260[8] = 0;
    game.f_05ee();
    assert_eq!(game.restart_stage, 1);
    assert_eq!(game.objects[52].state, State::Inactive);
    assert_eq!(game.objects[FIGHTER].x, 94); // displayed X, not the buffer's 100
    assert_eq!(game.objects[FIGHTER].y, 0x1f8); // SUB low byte; no ninth-bit borrow
    assert_eq!(game.objects[FIGHTER].state, State::Rogue);
    assert_eq!(game.main_task_enable[20], 0);
    assert_eq!(game.sub_task_enable[5], 0);
    assert_eq!(game.combat.sound_requests[0x19], 2);
}

#[test]
fn fighter_hit_resumes_at_original_rocket_scan_tail() {
    let mut game = Game::default();
    game.main_task_enable[8] = 0;
    game.objects[FIGHTER].x = 100;
    game.objects[FIGHTER].y = 200;
    rest(&mut game, 8, 3);
    rest(&mut game, 10, 3);
    game.objects[4].y = 200;
    game.objects[5].y = 204;
    game.objects[5].x = 105;
    game.motions[0].flags = 1;
    assert!(game.hitd_fghtr_notif(FIGHTER));
    assert_eq!(game.combat.hit_notifications[4], 0x81);
    assert_eq!(game.combat.hit_notifications[5], 0x81);
    assert_eq!(game.combat.hits, 1); // first collision does not increment, tail rocket hit does
    assert_eq!(game.objects[FIGHTER].y, 200); // even impactor register clears X
}

#[test]
fn original_life_flow_waits_and_respawns_without_resetting_enemies() {
    let mut game = Game::default();
    game.restart_stage = 1;
    game.active_enemy_count = 7;
    game.main_task_enable[8] = 0;
    game.main_task_enable[20] = 0;
    game.main_task_enable[21] = 0;
    game.sub_task_enable[5] = 0;
    game.background_run();
    assert_eq!(game.background, Background::RestartWait);
    assert_eq!(game.game_timers[3], 4);
    game.flying_count = 2;
    game.game_timers[3] = 0;
    game.background_run();
    assert_eq!(game.reserve_fighters, 1);
    assert_eq!(game.background, Background::RespawnWaitForBugs);
    assert_eq!(game.main_task_enable[21], 0);
    game.flying_count = 0;
    game.background_run();
    assert_eq!(game.objects[FIGHTER].x, 122);
    assert_eq!(game.background, Background::RespawnDelay);
    assert_eq!(game.restart_stage, 0);
    assert_eq!(game.active_enemy_count, 7);
    assert_eq!(game.game_timers[3], 3);
    game.game_timers[3] = 0;
    game.background_run();
    assert_eq!(game.background, Background::GameRunner);
    assert_eq!(game.main_task_enable[21], 1);
    assert_eq!(game.sub_task_enable[5], 1);
}

#[test]
fn text_and_ratio_preserve_original_digits_and_tile_walk() {
    let mut game = Game::default();
    for value in 0..=65535 {
        let (q, r) = Game::c_divmod(value, 10);
        assert_eq!((q, r), (value / 10, (value % 10) as u8));
    }
    game.c_sctrl_playfld_clr();
    let end = game.c_string_out(3, None);
    assert_eq!(end, 0x8190);
    assert_eq!([game.tile_ram[0x270], game.tile_ram[0x250], game.tile_ram[0x230]], [27,14,10]);
    for (hits, shots, expected) in [(0,0,"0.0"), (1,2,"50.0"), (1,3,"33.3"),
                                    (3,4,"75.0"), (1,1,"100.0"), (99,100,"99.0")] {
        game.c_sctrl_playfld_clr();
        game.combat.hits = hits;
        game.shots = shots;
        let end = game.c_0a72_puts_hitmiss_ratio();
        let mut actual = String::new();
        let mut address = 0x8138u16;
        while address != end {
            let tile = game.tile_ram[(address & 0x3ff) as usize];
            actual.push(if tile == 0x2a { '.' } else { char::from(b'0' + tile) });
            address -= 32;
        }
        assert_eq!(actual, expected, "{hits}/{shots}");
    }
}

#[test]
fn object_bounds_read_copied_sprite_registers_before_pending_buffer() {
    let mut game = Game::default();
    game.main_task_enable.fill(0);
    game.main_task_enable[12] = 1;
    game.counters[0] = 1;
    game.objects[4] = Object { state: State::Bomb, x: 255, y: 400, ..Object::default() };
    game.sprite_ram[4].x = 100;
    game.sprite_ram[4].y = 100;
    game.cpu0_rst38();
    assert_eq!(game.objects[4].state, State::Bomb);
    game.sprite_ram[4].x = 255;
    game.cpu0_rst38();
    assert_eq!(game.objects[4].state, State::Inactive);
    assert_eq!(game.objects[4].x, 0);
}
