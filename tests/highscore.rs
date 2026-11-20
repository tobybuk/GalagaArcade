use galaga_reconstructed::{executive::Background,Game};

fn digits(mut score:u32)->[u8;6] {
    let mut d=[0x24;6];
    for (i, digit) in d.iter_mut().enumerate() {
        if score!=0 || i<2 { *digit=(score%10) as u8; score/=10; }
    }
    d
}
fn setup(score:u32)->Game {
    let mut game=Game::default();
    for (rank,score) in [100000,80000,60000,40000,20000].into_iter().enumerate() {
        game.top5_ram[0x20+rank*6..0x26+rank*6].copy_from_slice(&digits(score));
        game.top5_ram[0x3e+rank*3..0x41+rank*3].copy_from_slice(&[10+rank as u8;3]);
    }
    game.tile_ram[0x3f8..0x3fe].copy_from_slice(&digits(score));
    game.c_sctrl_playfld_clr();
    game
}

#[test]
fn original_high_score_insertions_preserve_above_rows_and_shift_below() {
    for (score,placement) in [(110000,1),(90000,2),(70000,3),(50000,4),(30000,5)] {
        let mut game=setup(score);
        let previous=game.top5_ram;
        game.c_top5_dlg_proc();
        assert_eq!(game.background,Background::Top5IntroWait);
        assert_eq!(game.top5_ram[0x11],placement);
        let at=placement as usize-1;
        for row in 0..5 {
            let expected=if row<at { &previous[0x20+row*6..0x26+row*6] }
                else if row==at { &digits(score)[..] }
                else { &previous[0x20+(row-1)*6..0x26+(row-1)*6] };
            assert_eq!(&game.top5_ram[0x20+row*6..0x26+row*6],expected);
        }
        assert_eq!(&game.top5_ram[0x3e+at*3..0x41+at*3],&[0x24;3]);
        assert_eq!(game.tile_ram[0x149],10);
        assert_eq!(game.game_timers[2],4);
        assert_eq!(game.colour_ram[0x374+at*2],5);
        if placement==1 { assert_eq!(game.combat.sound_requests[12],0xff); }
        else { assert_eq!(game.combat.sound_requests[16],placement); }
    }
}

#[test]
fn equal_score_does_not_displace_original_fifth_place() {
    let mut game=setup(20000);
    game.c_top5_dlg_proc();
    assert_eq!(game.background,Background::GameOverSoundWait);
    assert_eq!(game.top5_ram[0x11],0);
}

#[test]
fn initials_commit_uses_original_three_positions_and_frame_reset() {
    let mut game=setup(110000);game.c_top5_dlg_proc();
    game.game_timers[2]=0;game.background_run();
    assert_eq!(game.background,Background::Top5Entry);
    for (column,letter) in [(0x149,11),(0x129,12),(0x109,13)] {
        game.tile_ram[column]=letter;
        game.io_input[1]=0x2f;
        game.counters[0]=game.counters[0].wrapping_add(1);
        game.background_run();
    }
    assert_eq!(&game.top5_ram[0x3e..0x41],&[11,12,13]);
    assert_eq!(game.background,Background::Top5FinishWait);
    assert_eq!(game.counters[0],76);
    assert_eq!(game.tile_ram[0x134],11); // score template: six score columns, six-space gap, initials
    assert_eq!(game.tile_ram[0x114],12);
    assert_eq!(game.tile_ram[0xf4],13);
    game.counters[0]=0;game.background_run();
    assert_eq!(game.background,Background::GameOverSoundWait);
}

#[test]
fn initials_timeout_and_coin_in_commit_remaining_displayed_characters() {
    for coin_in in [false,true] {
        let mut game=setup(110000);game.c_top5_dlg_proc();
        game.game_timers[2]=0;game.background_run();
        game.tile_ram[0x149]=35;game.tile_ram[0x129]=36;game.tile_ram[0x109]=42;
        if coin_in { game.io_input[0]=1; }
        else { game.game_timers[2]=0; }
        game.background_run();
        assert_eq!(&game.top5_ram[0x3e..0x41],&[35,36,42]);
        assert_eq!(game.background,Background::GameOverSoundWait);
    }
}

#[test]
fn initials_selection_keeps_original_space_dot_and_letter_wrap() {
    let mut game=setup(110000);game.c_top5_dlg_proc();
    game.game_timers[2]=0;game.background_run();
    game.io_input[1]=0x18;game.top5_ram[2]=8;
    for (before,after) in [(35,36),(36,42),(42,10)] {
        game.tile_ram[0x149]=before;game.top5_ram[3]=15;
        game.counters[0]=game.counters[0].wrapping_add(1);
        game.background_run();assert_eq!(game.tile_ram[0x149],after);
    }
}
