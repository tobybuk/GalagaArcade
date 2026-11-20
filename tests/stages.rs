use galaga_reconstructed::{executive::{Background, StageAfter}, Game, Object, State, FIGHTER};

#[test]
fn all_stage_rank_wave_tables_keep_original_fixed_objects_and_pair_structure() {
    for rank in 0..4 {
        for stage in 1..=255 {
            let mut game = Game::default();
            game.machine_rank = rank; game.stage_number = stage;
            game.initialise_enemy_classes();
            game.gctl_stg_new_atk_wavs_init();
            assert_eq!(game.fault, None, "rank {rank}, stage {stage}");
            let mut cursor = 0;
            let mut waves = 0;
            let mut fixed = Vec::new();
            while game.wave_sequence[cursor] != 0x7f {
                assert_eq!(game.wave_sequence[cursor], 0x7e);
                waves += 1; cursor += 1;
                let mut count = 0;
                while ![0x7e,0x7f].contains(&game.wave_sequence[cursor]) {
                    assert!(game.wave_sequence[cursor] & 0x3f < 24);
                    let id = game.wave_sequence[cursor + 1];
                    if id & 0x38 != 0x38 { fixed.push(id); }
                    cursor += 2; count += 1;
                }
                assert!((8..=16).contains(&count) && count & 1 == 0);
                assert!(cursor < 192);
            }
            assert_eq!(waves, 5, "rank {rank}, stage {stage}");
            fixed.sort_unstable(); fixed.dedup();
            assert_eq!(fixed.len(), 40, "rank {rank}, stage {stage}");
            assert!(fixed.iter().all(|&id| id < 0x60 && id & 1 == 0));
        }
    }
}

#[test]
fn challenge_classes_and_score_attributes_follow_original_rotations() {
    for (stage, packed, attributes) in [(3,0x36,[10,0xb8]),(7,0x24,[10,0xb8]),
        (11,0xd4,[15,0xb9]),(19,0xe4,[20,0xbc]),(27,0xa8,[30,0xbd]),(35,0x36,[30,0xbd])] {
        let mut game = Game::default(); game.stage_number = stage;
        game.initialise_enemy_classes();
        assert_eq!(game.combat.challenge_attributes, attributes);
        assert_eq!(game.objects[4].code & 0x7f, packed >> 1);
        assert_eq!(game.objects[32].code & 0x7f, packed >> 1);
        assert_eq!(game.objects[24].code & 0x7f, 8);
    }
}

#[test]
fn transient_class_and_object_remap_follow_original_launch_branches() {
    for (id, wave, code, colour) in [(0x3a,1,0x18,3),(0x7a,1,0x10,2),(0x3a,2,8,0)] {
        let mut game = Game::default();
        game.wave_sequence[..2].copy_from_slice(&[0,id]);
        game.wave_cursor = 0; game.wave_number = wave;
        game.wave_task();
        assert_eq!((game.objects[29].code,game.objects[29].colour),(code,colour));
        assert_eq!(game.objects[29].state,State::Spawning);
        assert_eq!(game.motions[0].object_id,0x3a);
        assert_eq!(game.motions[0].bomb_flags,0);
    }
}

#[test]
fn challenge_marker_waits_for_original_timer_and_reloads_hit_eight_counter() {
    let mut game = Game::default(); game.stage_number = 3;
    game.game_timers[0] = 2;
    game.wave_task(); assert_eq!(game.wave_cursor,0);
    game.game_timers[0] = 1;
    game.wave_task(); assert_eq!(game.wave_cursor,0);
    assert_eq!(game.combat.flight_hits_remaining,8);
    game.game_timers[0] = 0;
    game.wave_task(); assert_eq!(game.wave_cursor,1);
}

#[test]
fn stage_icons_keep_original_eight_frame_wait_and_rev_b_reserve_protection() {
    let mut game = Game::default(); game.stage_number = 1;
    game.reserve_fighters = 8; game.draw_resv_ships();
    assert_eq!(game.tile_ram[0x13],0x4a);
    game.counters[0] = 252;
    game.c_new_level_tokens(2,false);
    assert_eq!(game.tile_ram[0x13],0x4a);
    assert!(!game.stage_tokens_continue());
    for frame in [253,254,255,0,1,2,3] { game.counters[0]=frame; assert!(!game.stage_tokens_continue()); }
    game.counters[0]=4;
    assert!(game.stage_tokens_continue());
    assert_eq!(game.tile_ram[2],0x36); assert_eq!(game.tile_ram[0x22],0x37);
    assert_eq!(game.colour_ram[2],1);
    assert_eq!(game.combat.sound_requests[21],2);
    assert_eq!(game.tile_ram[0x13],0x4a);
}

#[test]
fn a_challenge_round_runs_all_five_waves_and_advances_to_next_stage() {
    let mut game = Game::default();
    game.c_sctrl_sprite_ram_clr();
    game.objects[FIGHTER] = Object { x:122,y:297,code:6,colour:9,..Object::default() };
    game.main_task_enable[8]=0;
    game.stage_number=2;
    game.stg_init_splash(StageAfter::Ready);
    let mut saw_completed_challenge=false;
    for _ in 0..12000 {
        game.step(0);
        assert_eq!(game.fault,None,"stage {}, background {:?}",game.stage_number,game.background);
        if game.stage_number==3 && game.entrance_complete { saw_completed_challenge=true; }
        if game.stage_number==4 && game.background==Background::GameRunner { break; }
    }
    assert!(saw_completed_challenge);
    assert_eq!(game.stage_number,4);
    assert_eq!(game.background,Background::GameRunner);
    assert_eq!(game.reserve_fighters,2);
    assert_eq!(game.score_value(0),0);
}

#[test]
fn original_challenge_awards_each_hit_or_the_perfect_bonus() {
    for (hits, expected) in [(0,0),(23,2300),(40,10000)] {
        let mut game = Game::default(); game.stage_number=3;
        game.combat.flying_hits=hits;
        game.active_enemy_count=0; game.main_task_enable[8]=0;
        game.background_run();
        for frame in 0..500 {
            game.game_timers[3]=0; game.counters[0]=frame as u8;
            game.background_run();
            if game.background==Background::ChallengeAwardWaitOne { break; }
        }
        assert_eq!(game.score_value(0),expected,"hits {hits}");
        assert_eq!(game.background,Background::ChallengeAwardWaitOne);
    }
}

#[test]
fn refresh_lookup_uses_rom_page_and_both_original_samples() {
    let game=Game::default();
    for first in 0..=255 {
        for second in [0,1,127,128,255] {
            assert_eq!(game.c_1000_samples(first,second),
                galaga_reconstructed::rom_data::REFRESH_SAMPLE_PAGE[first as usize].wrapping_add(second));
        }
    }
}

#[test]
fn stage_counter_wrap_uses_original_rotate_and_adjacent_rom_reads() {
    use galaga_reconstructed::{formation::stage_parameters, rom_data::*};
    for (rank,row) in [(0,1),(1,2),(2,3),(3,0)] {
        let parameters=stage_parameters(0,rank);
        for i in 0..5 {
            // DEC 0 -> FF; two RLCA leave FF; ADD FF -> FE, not FB.
            let index=row*130+254+i;
            let value=if index<520 { STAGE_PARAMETERS[index] } else { STAGE_PARAMETER_WRAP_TAIL[index-520] };
            assert_eq!(&parameters[i*2..i*2+2],&[value>>4,value&15]);
        }
        let mut game=Game::default(); game.stage_number=0; game.machine_rank=rank;
        game.c_2c00(); game.f_0857();
        assert_eq!(game.star_control[2],0x40);
    }
}
