use galaga_reconstructed::{executive::Background,Game};

fn service()->Game {
    let mut g=Game::power_on();
    for frame in 0..32 {
        g.step_ports([15,15,15,7]);assert_eq!(g.fault,None,"entry {frame}");
        if g.background==Background::ServiceMenu {
            for _ in 0..4 {g.step_ports([15,15,15,7]);}
            return g;
        }
    }
    panic!("Service did not reach original menu: {:?}",g.background);
}
fn hold(g:&mut Game,stick:u8,buttons:u8,coins:u8,frames:u16) {
    for frame in 0..frames {g.step_ports([stick,15,buttons,coins]);assert_eq!(g.fault,None,"frame {frame}");}
}

#[test]
fn service_switch_runs_original_sound_selection_and_returns_through_crosshatch() {
    let mut g=service();assert!(g.service_active);
    assert_eq!(g.main_task_enable[0],32);assert_eq!(g.sub_task_enable[0],7);
    assert_eq!(g.shared_9260[16],0);
    hold(&mut g,13,15,7,4);hold(&mut g,15,15,7,4);
    assert_eq!(g.shared_9260[16],1);assert_eq!(g.tile_ram[0x20e],1);
    hold(&mut g,7,15,7,4);hold(&mut g,15,15,7,4);
    assert_eq!(g.shared_9260[16],0);
    g.total_plays_bcd=[0,0x12];g.total_score_bcd=[1,2,3,4];
    hold(&mut g,15,15,15,14);assert_eq!(g.background,Background::ServiceCrossWait);
    assert_eq!(&g.tile_ram[..4],&[0x28,0x27,0x28,0x27]);
    assert_eq!(&g.tile_ram[32..36],&[0x2d,0x2b,0x2d,0x2b]);
    hold(&mut g,15,15,7,140);assert_eq!(g.background,Background::ServiceCrossHold);
    hold(&mut g,15,15,15,12);
    assert!(!g.service_active);assert_eq!(g.game_state,1);
    assert_eq!(g.background,Background::AttractWait);
    assert_eq!(g.total_plays_bcd,[0,0x12]);assert_eq!(g.total_score_bcd,[1,2,3,4]);
}

#[test]
fn service_machine_totals_keep_original_complements_points_and_timed_erase() {
    let mut g=service();
    g.total_plays_bcd=[1,0x23];g.total_score_bcd=[0,0x12,0x34,0x56];
    g.game_time_bcd=[0,1,0x23,0x45];g.bonus_count_bcd=[1,0x23];
    g.c_svc_test_input_hdlr_39e0(15);
    let tiles:Vec<_>=(0..23).map(|i|g.tile_ram[0x35e-i*32]).collect();
    let expected:Vec<_>="87.6987.6543.9987.6876.".chars().map(|c|if c=='.' {0x2a} else {c as u8-b'0'}).collect();
    assert_eq!(tiles,expected);
    assert_eq!(&g.shared_9260[18..20],&900u16.to_le_bytes());
    hold(&mut g,15,15,7,901);
    assert!((0..23).all(|i|g.tile_ram[0x35e-i*32]==0x24));
}

#[test]
fn service_original_fire_and_5r_6l_3r_7l_sequence_draws_supplied_easter_egg() {
    let mut g=service();hold(&mut g,15,14,7,8);
    let sequence=[(13,5),(7,6),(13,3),(7,7)];let mut count=0;
    for (stick,repetitions) in sequence {
        for _ in 0..repetitions {
            hold(&mut g,stick,14,7,4);hold(&mut g,15,14,7,4);count+=1;
            assert_eq!(g.shared_9260[17],count);
        }
    }
    assert_eq!(g.background,Background::ServiceEasterEgg);
    assert!(g.tile_ram.iter().any(|tile|*tile==0x25));
    assert!(g.tile_ram.iter().all(|tile|*tile==0x24 || *tile==0x25));
}

#[test]
fn unused_dip_follows_original_sub_reset_instead_of_an_untranslated_stop() {
    let mut g=Game::default();g.unused_dsw3=0;g.sub_task_enable=[0;8];
    g.sub_diagnostic[0]=9;g.f_0eca();
    assert_eq!(g.fault,None);assert_eq!(g.sub_diagnostic[0],0);
    assert_eq!(g.sub_task_enable,[0,1,1,0,1,1,0,10]);
}

#[test]
fn original_external_editor_keeps_fifo_nibble_order_target_access_and_selection() {
    use std::collections::BTreeMap;
    let mut g=Game::default();g.sub_diagnostic[2..4].copy_from_slice(&0x1000u16.to_le_bytes());
    let mut ram=BTreeMap::from([(0x1000,0x42),(0x1001,0x53),(0xe000,0x64)]);
    let mut memory=|address,value|match value {
        Some(value)=>{ram.insert(address,value);value},None=>*ram.get(&address).unwrap_or(&0xff),
    };
    for _ in 0..4 {g.sub_editor_0ede([255;5],&mut memory);}
    assert_eq!(&g.tile_ram[0x3ca..0x3d0],&[2,4,0,0,0,1]);
    fn pulse(g:&mut Game,group:usize,bit:u8,memory:&mut impl FnMut(u16,Option<u8>)->u8) {
        let mut ports=[255;5];ports[group]&=!bit;
        for _ in 0..2 {g.sub_editor_0ede(ports,memory);}
        for _ in 0..2 {g.sub_editor_0ede([255;5],memory);}
    }
    pulse(&mut g,3,8,&mut memory); // data low nibble = 3
    assert_eq!(memory(0x1000,None),0x43);assert_eq!(g.sub_diagnostic[0],1);
    pulse(&mut g,3,2,&mut memory); // data high nibble = 1
    assert_eq!(memory(0x1000,None),0x13);assert_eq!(g.sub_diagnostic[0],0);
    pulse(&mut g,4,8,&mut memory); // address increment
    assert_eq!(&g.sub_diagnostic[1..4],&[0x53,1,0x10]);
    pulse(&mut g,4,2,&mut memory); // address decrement
    assert_eq!(&g.sub_diagnostic[1..4],&[0x13,0,0x10]);
    pulse(&mut g,4,1,&mut memory);assert_eq!(g.sub_diagnostic[0],5);
    pulse(&mut g,0,4,&mut memory); // address upper nibble = E
    assert_eq!(&g.sub_diagnostic[..4],&[4,0x64,0,0xe0]);
    assert_eq!(&g.colour_ram[0x3ca..0x3d0],&[1,1,1,1,0,1]);
}
