use galaga_reconstructed::Game;

#[test]
fn original_dip_wiring_decodes_all_life_bonus_coin_and_rank_selections() {
    let standard=[[255,255],[3,10],[2,7],[2,134],[2,6],[3,12],[2,8],[3,136]];
    let five=[[255,255],[3,138],[3,12],[3,143],[3,10],[3,140],[3,15],[3,255]];
    let coinage=[[0,0],[2,3],[3,1],[1,2],[4,1],[1,3],[2,1],[1,1]];
    let mut game=Game::default();
    for switches in 0..=255u8 {
        game.configure_machine(0xf7,switches);
        let reserves=[1,3,2,4][(switches>>6) as usize];
        assert_eq!(game.machine_reserves,reserves);
        assert_eq!(game.machine_bonus,if reserves==4 {five} else {standard}[((switches>>3)&7) as usize]);
        let coins=coinage[(switches&7) as usize];
        assert_eq!(&game.io_parameters[1..5],&[coins[0],coins[1],coins[0],coins[1]]);
    }
    for switches in 0..=255u8 {
        game.configure_machine(switches,0x97);
        assert_eq!(game.machine_rank,switches&3);
        assert_eq!(game.machine_cabinet,u8::from(switches&128==0));
        assert_eq!(game.machine_dsw5&2,((switches>>4)&1)<<1);
    }
}

#[test]
fn production_boot_uses_original_configuration_and_rev_b_fighters_text() {
    let game=Game::power_on();
    assert_eq!((game.machine_rank,game.machine_reserves,game.machine_cabinet),(3,2,0));
    assert_eq!(game.machine_bonus,[2,7]);
    let mut display=Game::default();display.configure_machine(0xf7,0x97);
    let shown:(Vec<_>,Vec<_>)=((0..8).map(|i|display.tile_ram[0x2aa-i*32]).collect(),
        (0..7).map(|i|display.tile_ram[0x2e6-i*32]).collect());
    assert_eq!(shown.0,[15,18,16,17,29,14,27,28]); // FIGHTERS in the supplied revision
    assert_eq!(shown.1,[30,25,27,18,16,17,29]); // UPRIGHT
}

#[test]
fn original_freeze_switch_stops_both_task_kernels_and_frame_dividers() {
    let mut game=Game::power_on_with_switches(0xe7,0x97);
    let counters=game.counters;
    for _ in 0..32 {game.step(0);}
    assert_eq!(game.fault,None);assert_eq!(game.counters,counters);
    assert_eq!(game.main_task_trace_len,0);assert_eq!(game.star_port_latch&7,7);
    assert_eq!(game.elapsed_frames,32);
    game.configure_machine(0xf7,0x97);game.step(0);
    assert_eq!(game.counters[0],1);assert!(game.main_task_trace_len>0);
}
