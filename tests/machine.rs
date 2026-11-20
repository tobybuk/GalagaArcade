use galaga_reconstructed::{executive::Background,Game,FIGHTER};

#[test]
fn production_cold_boot_keeps_original_test_configuration_and_crosshatch_route() {
    let mut g=Game::cold_boot_with_switches(0xf7,0x97);
    assert!(g.service_active);assert_eq!(g.background,Background::ServiceWaitTwo);
    let ram:Vec<_>=(0..7).map(|i|g.tile_ram[0x2e2-i*32]).collect();
    let rom:Vec<_>=(0..7).map(|i|g.tile_ram[0x2e4-i*32]).collect();
    assert_eq!(ram,[0x1b,0x0a,0x16,0x24,0x24,0x18,0x14]);
    assert_eq!(rom,[0x1b,0x18,0x16,0x24,0x24,0x18,0x14]);
    let mut crosshatch=false;
    for frame in 0..400 {
        g.step(0);assert_eq!(g.fault,None,"frame {frame}");
        crosshatch|=matches!(g.background,Background::ServiceCrossWait|Background::ServiceCrossHold);
        if !g.service_active {break;}
    }
    assert!(crosshatch);assert!(!g.service_active);
    assert_eq!(g.game_state,1);assert_eq!(g.background,Background::AttractWait);
}

#[test]
fn boot_credit_debit_and_original_opening_delays_reach_stage_one() {
    let mut g=Game::power_on();
    assert_eq!(g.game_state,1); assert_eq!(g.background,Background::AttractWait);
    g.step(8);g.step(8);g.step(0);g.step(0);
    assert_eq!(g.game_state,2); assert_eq!(g.credit_count,1);
    assert_eq!(g.background,Background::ReadyWait);
    g.step(16);g.step(16);g.step(0);g.step(0);
    assert_eq!(g.game_state,3); assert_eq!(g.credit_count,0);
    assert_eq!(g.background,Background::OpeningThemeWait);
    assert_eq!(g.stage_number,0); assert_eq!(g.objects[FIGHTER].x,0);
    let mut count=0;
    while g.background!=Background::GameRunner && count<600 {
        g.step(0);count+=1;
        assert_eq!(g.fault,None,"after {count}");
    }
    assert_eq!(g.background,Background::GameRunner);
    assert_eq!(g.stage_number,1); assert_eq!(g.reserve_fighters,2);
    assert!(count>250); assert_eq!(g.objects[FIGHTER].x,0x7a);
    assert_eq!(g.suspended_player[0x20],1);
}

#[test]
fn all_original_attract_states_and_demonstrations_cycle_without_stopping() {
    let mut g=Game::power_on();
    let mut seen=[false;15]; let mut wraps=0; let mut previous=0;
    for frame in 0..24000 {
        seen[g.attract_state as usize]=true;
        g.step(0);
        assert_eq!(g.fault,None,"frame {frame}, attract {}, vector {:04x}",g.attract_state,g.demo_vector);
        if previous==14 && g.attract_state==0 {wraps+=1;}
        previous=g.attract_state;
        if wraps==2 {break;}
    }
    assert!(seen.iter().all(|v|*v),"seen {seen:?}, stopped at {}, vector {:04x}, timer {}, task {}",g.attract_state,g.demo_vector,g.demo_timer,g.main_task_enable[3]);
    assert_eq!(wraps,2);
}

#[test]
fn original_credit_task_renders_bcd_and_preserves_coin_sound_slot() {
    let mut g=Game::power_on(); g.combat.sound_requests.fill(7);
    g.io_input[0]=0x12; g.f_0977();
    assert_eq!(g.credit_count,0x12); assert_eq!(g.coin_sound_count,0x12);
    assert_eq!(g.game_state,2); assert_eq!(g.combat.sound_requests[8],7);
    assert_eq!(g.tile_ram[0x35],1); assert_eq!(g.tile_ram[0x34],2);
    g.io_input[0]=0x10; g.f_0977();
    assert!(g.two_player_game); assert_eq!(g.game_state,3);
    g.game_time_bcd=[0,0,0x59,0x59];g.f_0977();
    assert_eq!(g.game_time_bcd,[0,0,0x60,0]);
}

#[test]
fn game_recycle_keeps_top5_and_accumulates_original_bcd_machine_totals() {
    let mut g=Game::default();
    let top=g.top5_ram;
    g.total_plays_bcd=[0x99,0x99];
    g.total_score_bcd=[0,0x99,0x99,0x99];
    g.tile_ram[0x3f9..0x3fe].copy_from_slice(&[1,2,3,4,5]);
    g.tile_ram[0x3e4..0x3e9].fill(0x24);
    g.g_halt();g.background_run();
    assert_eq!(g.background,Background::HaltIoComplete);
    g.background_run();
    assert_eq!(g.total_plays_bcd,[0,0]);
    assert_eq!(g.total_score_bcd,[1,5,0x43,0x20]); // accumulated units of ten points
    assert_eq!(g.background,Background::AttractWait);
    assert_eq!(g.top5_ram,top); assert_eq!(g.game_state,1);
}

#[test]
fn attract_fighter_wait_resumes_the_same_irq_call_without_replaying_sprite_setup() {
    let mut g=Game::power_on();g.attract_state=3;g.sprite_tiles_pointer=0x195c;
    g.flying_count=1;g.step(0);
    assert_eq!(g.fault,None);assert!(g.attract_setup_pending);
    let pointer=g.sprite_tiles_pointer;
    assert_eq!(g.main_task_trace[g.main_task_trace_len-1],0x17b2);
    assert_eq!(g.flying_count,0); // independent sub board completed the wait
    g.step(0);
    assert_eq!(g.fault,None);assert!(!g.attract_setup_pending);
    assert_eq!(g.sprite_tiles_pointer,pointer);assert_eq!(g.attract_state,4);
    assert_eq!(g.main_task_trace[0],0x17b2);
}
