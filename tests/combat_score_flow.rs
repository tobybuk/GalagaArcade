use galaga_reconstructed::{Game, Object, State, FIGHTER, ROCKET_0};

fn target(game: &mut Game, colour: u8, state: State) {
    game.objects[4] = Object { state, colour, code: 0x18, x: 100, y: 100, motion_slot: 0, ..Object::default() };
    game.motions[0].flags = 1;
    game.objects[ROCKET_0] = Object { x: 100, y: 100, colour: 9, code: 0x30, ..Object::default() };
}

#[test]
fn green_boss_first_hit_changes_colour_and_consumes_y_only() {
    let mut game = Game::default();
    target(&mut game, 0, State::Standby);
    game.hitd_det_rckt(ROCKET_0);
    assert_eq!(game.combat.hits, 1);
    assert_eq!(game.objects[4].colour, 1);
    assert_eq!(game.objects[ROCKET_0].x, 100);
    assert_eq!(game.objects[ROCKET_0].y, 0);
    assert_eq!(game.combat.hit_notifications[4], 0);
    assert_eq!(game.combat.hit_multipliers, [0; 16]);
    assert_eq!(game.combat.sound_requests[4], 1);
}

#[test]
fn stationary_and_flying_points_follow_original_colour_factors() {
    for (colour, state, points) in [(2, State::Standby, 80), (3, State::Standby, 50),
                                    (1, State::Standby, 150), (2, State::Pattern, 160),
                                    (3, State::Pattern, 100), (1, State::Pattern, 400)] {
        let mut game = Game::default();
        target(&mut game, colour, state);
        game.hitd_det_rckt(ROCKET_0);
        game.gctl_supv_score();
        assert_eq!(game.score_value(0), points, "colour={colour},state={state:?}");
    }
}

#[test]
fn two_rockets_cannot_award_the_same_fatal_hit_twice() {
    let mut game = Game::default();
    target(&mut game, 3, State::Pattern);
    game.objects[ROCKET_0 + 1] = game.objects[ROCKET_0];
    game.hitd_det_rckt(ROCKET_0);
    game.hitd_det_rckt(ROCKET_0 + 1);
    assert_eq!(game.combat.hits, 1);
    assert_eq!(game.combat.hit_notifications[4], 0x81);
    assert_eq!(game.combat.hit_multipliers[3], 2);
    assert_eq!(game.motions[0].flags, 0);
}

#[test]
fn bullet_rectangle_preserves_unsigned_carry_boundaries() {
    for dx in -8i16..=8 {
        for dy in -10i16..=10 {
            let mut game = Game::default();
            target(&mut game, 3, State::Standby);
            game.objects[4].x = (100 + dx) as u8;
            game.objects[4].y = (100 + dy) as u16;
            game.hitd_det_rckt(ROCKET_0);
            let scaled = ((100 + dy) >> 1) - 50;
            let expected = (-5..=5).contains(&dx) && (-3..=2).contains(&scaled);
            assert_eq!(game.combat.hits != 0, expected, "dx={dx},dy={dy}");
        }
    }
}

#[test]
fn hit_counter_decrements_only_the_original_low_byte() {
    let mut game = Game::default();
    target(&mut game, 3, State::Pattern);
    game.hitd_det_rckt(ROCKET_0);
    assert_eq!(game.combat.flight_hits_remaining, 255);
    game.combat.flight_hits_remaining = 1;
    game.combat.challenge_attributes = [10, 0xb8];
    game.combat.hit_notifications[4] = 0;
    game.objects[ROCKET_0].y = 100;
    game.hitd_det_rckt(ROCKET_0);
    assert_eq!(game.combat.flight_hits_remaining, 0);
    assert_eq!(game.combat.hit_notifications[4], 0xb8);
    assert_eq!(game.combat.hit_multipliers[15], 10);
}

#[test]
fn original_explosion_sequence_runs_through_notification_and_jump_table() {
    let mut game = Game::default();
    target(&mut game, 3, State::Standby);
    game.hitd_det_rckt(ROCKET_0);
    assert_eq!(game.objects[4].state, State::Standby);
    game.f_1db3();
    assert_eq!(game.objects[4].state, State::Exploding);
    assert_eq!(game.objects[4].motion_slot, 0x40);
    assert_eq!(game.combat.hit_notifications[4], 1);
    for code in [0x41, 0x42, 0x43, 0x44, 0x48] {
        game.case_24b2(4);
        assert_eq!(game.objects[4].code, code);
    }
    assert_eq!(game.objects[4].x, 92);
    assert_eq!(game.objects[4].y, 92);
    assert_eq!(game.objects[4].flags, 12);
    game.case_24b2(4);
    assert_eq!(game.objects[4].state, State::Inactive);
    assert_eq!(game.objects[4].x, 0);
}

#[test]
fn moving_boss_uses_score_attribute_bytes_and_bonus_bitmap_lifetime() {
    let mut game = Game::default();
    // Boss ID 0x30 uses attribute bytes 0 and 1. This does not depend on its
    // motion-pool slot (a bug in the older C translation).
    let mut boss = Object { state: State::Homing, colour: 1, code: 8, x: 100, y: 100,
                            motion_slot: 9, ..Object::default() };
    game.objects[24] = boss;
    game.motions[9].flags = 1;
    game.objects[ROCKET_0] = Object { x: 100, y: 100, ..Object::default() };
    game.hitd_det_rckt(ROCKET_0);
    assert_eq!(game.combat.hit_multipliers[15], 1);
    assert_eq!(game.combat.hit_notifications[24], 0xb5);
    assert_eq!(game.game_timers[1], 6);
    game.gctl_supv_score();
    assert_eq!(game.score_value(0), 400);
    game.f_1db3();
    for _ in 0..6 { game.case_24b2(24); }
    boss = game.objects[24];
    assert_eq!(boss.state, State::Score);
    assert_eq!(boss.code, 0x35);
    assert_eq!(boss.motion_slot, 19);
    for _ in 0..19 { game.case_2535(24); }
    assert_eq!(game.objects[24].state, State::Inactive);
}

#[test]
fn rocket_312_boundary_is_exclusive_as_in_the_rom() {
    let mut game = Game::default();
    game.objects[ROCKET_0].x = 100;
    game.objects[ROCKET_0].y = 306;
    game.rocket_attributes[0] = 0x80; // +6 vertical
    game.rocket_motion();
    assert_eq!(game.objects[ROCKET_0].y, 312);
    assert_eq!(game.objects[ROCKET_0].x, 0);
}

#[test]
fn score_ram_carries_and_high_score_follow_the_original_writes() {
    let mut game = Game::default();
    for _ in 0..200 { game.combat.hit_multipliers[15] = 1; game.gctl_supv_score(); }
    assert_eq!(game.score_value(0), 20000);
    assert_eq!(game.reserve_fighters, 3);
    assert_eq!(game.next_bonus, 7);
    assert_eq!(game.bonus_count_bcd[1], 1);
    game.combat.hit_multipliers[15] = 1;
    game.gctl_supv_score();
    assert_eq!(game.score_value(0), 20100);
    assert_eq!(&game.tile_ram[0x3ed..0x3f3], &game.tile_ram[0x3f8..0x3fe]);
    for _ in 0..499 { game.combat.hit_multipliers[15] = 1; game.gctl_supv_score(); }
    assert_eq!(game.score_value(0), 70000);
    assert_eq!(game.reserve_fighters, 4);
    assert_eq!(game.next_bonus, 14);
    assert_eq!(game.bonus_count_bcd[1], 2);
}

#[test]
fn scheduler_uses_original_vectors_and_saved_step_when_task_disables_itself() {
    let mut game = Game::default();
    game.main_task_enable.fill(0);
    game.main_task_enable[8] = 2;
    game.main_task_enable[9] = 1;
    game.main_task_enable[10] = 1;
    game.wave_cursor = 85;
    game.flying_count = 0;
    game.cpu0_rst38();
    assert_eq!(game.main_task_trace[0], 0x2916);
    assert!(!game.main_task_trace[..game.main_task_trace_len].contains(&0x1dec));
    assert_eq!(game.main_task_enable[8], 0);
    assert_eq!(game.fault, None);
    assert!(game.main_task_trace[..game.main_task_trace_len].contains(&0x1b6b));
}

#[test]
fn original_sprite_copy_halves_preserve_order_and_buffer_delay() {
    let mut game = Game::default();
    game.objects[4].x = 17;
    game.objects[FIGHTER].x = 66;
    game.f_0828();
    assert_eq!(game.sprite_ram[FIGHTER].x, 66);
    assert_eq!(game.sprite_ram[4].x, 0);
    game.objects[4].x = 18;
    game.objects[FIGHTER].x = 67;
    game.f_05bf();
    assert_eq!(game.sprite_ram[4].x, 18);
    assert_eq!(game.sprite_ram[FIGHTER].x, 66);
}

#[test]
fn score_is_consumed_by_background_runner_before_notification_task() {
    let mut game = Game::default();
    game.main_task_enable.fill(0);
    game.main_task_enable[11] = 1;
    game.main_task_enable[12] = 1;
    game.sub_task_enable.fill(0);
    game.counters[0] = 1;
    target(&mut game, 3, State::Standby);
    game.hitd_det_rckt(ROCKET_0);
    game.step(0);
    assert_eq!(game.score_value(0), 50);
    assert_eq!(game.objects[4].state, State::Exploding);
    assert_eq!(game.objects[4].code, 0x41);
}
