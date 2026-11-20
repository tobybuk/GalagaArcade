use galaga_reconstructed::{executive::Background,Game,Object,State};

#[test]
fn player_exchange_preserves_original_packed_enemy_format_and_task_tables() {
    let mut g=Game::default();
    g.stage_number=7;g.reserve_fighters=1;g.player_number=0;g.wave_cursor=31;
    g.combat.hits=432;g.shots=765;g.capture_motion_slot=3;g.next_bonus=14;
    g.active_player_ram[60]=0x57;
    g.suspended_player[0]=2;g.suspended_player[1]=11;g.suspended_player[32]=1;
    g.suspended_player[2..4].copy_from_slice(&0x8965u16.to_le_bytes());
    g.suspended_player[60]=0x28;
    g.objects[7]=Object {state:State::Standby,code:0x96,colour:3,x:100,y:222,flags:5,..Object::default()};
    g.objects[8]=Object {state:State::Exploding,code:0xb9,x:150,..Object::default()};
    g.suspended_objects[7]=0x91;g.suspended_objects[8]=0x30;
    g.reserve_task_enable[9]=6;g.main_task_enable[9]=7;
    g.c_player_active_switch();
    assert_eq!((g.stage_number,g.reserve_fighters,g.player_number,g.wave_cursor),(11,2,1,0x45));
    assert_eq!(g.active_player_ram[60],0x28);
    assert_eq!(g.suspended_player[60],0x57);
    assert_eq!((g.suspended_player[1],g.suspended_player[9],g.suspended_player[30]),(7,60,14));
    assert_eq!(&g.suspended_player[36..40],&[0xb0,1,0xfd,2]);
    assert_eq!(g.suspended_objects[7],0x93);assert_eq!(g.suspended_objects[8],0x39);
    assert_eq!((g.objects[7].code,g.objects[7].colour,g.objects[7].state),(0x16,1,State::Standby));
    assert_eq!((g.objects[7].x,g.objects[7].y,g.objects[7].flags),(100,222,5));
    assert_eq!((g.objects[8].code,g.objects[8].x,g.objects[8].state),(0x30,0,State::Inactive));
    assert_eq!((g.main_task_enable[9],g.reserve_task_enable[9]),(6,7));
    assert_eq!((g.main_task_enable[0],g.reserve_task_enable[0]),(0,0x1f));
}

#[test]
fn formation_retreat_and_return_keep_original_126_steps_and_ninth_bit() {
    for flip in [0,1] {
        let mut g=Game::default();g.flip_screen=flip;
        let rows=g.formation.coordinates;
        g.main_task_enable[14]=1;g.formation_change_timer=0;
        for _ in 0..126 {g.f_1d32();}
        assert_eq!(g.main_task_enable[14],1);g.f_1d32();assert_eq!(g.main_task_enable[14],0);
        for i in 10..16 {
            assert_eq!(g.formation.coordinates[i],if flip==0 {rows[i].wrapping_sub(126)&0x1ff} else {rows[i].wrapping_add(126)&0x1ff});
        }
        g.formation_change_timer=0x80;g.main_task_enable[14]=1;
        for _ in 0..127 {g.f_1d32();}
        assert_eq!(g.formation.coordinates,rows);assert_eq!(g.main_task_enable[14],0);
    }
}

#[test]
fn original_two_player_game_uses_both_sets_of_fighters_then_recycles() {
    let mut g=Game::power_on();
    for buttons in [8,8,0,0,8,8,0,0,32,32,0,0] {g.step(buttons);}
    assert!(g.two_player_game);assert_eq!(g.credit_count,0);
    let mut seen=[false;2];let mut changes=0;let mut previous=g.player_number;
    for frame in 0..40000 {
        if g.background==Background::GameRunner {seen[g.player_number as usize]=true;}
        g.step(0);
        assert_eq!(g.fault,None,"frame {frame}, background {:?}",g.background);
        if g.player_number!=previous {changes+=1;previous=g.player_number;}
        if g.total_plays_bcd[1]==2 {break;}
    }
    assert_eq!(seen,[true,true]);assert!(changes>=5,"changes {changes}");
    assert_eq!(g.total_plays_bcd,[0,2]);assert_eq!(g.game_state,1);
}
