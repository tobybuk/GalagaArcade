use galaga_reconstructed::{motion::Motion,Game,Object,State,FIGHTER,ROCKET_0};

#[test]
fn original_fire_checks_odd_control_bit_two_before_any_missile_stores() {
    let mut game=Game::default();
    game.objects[FIGHTER].write_odd_control(0xa5);
    game.launch_rocket();
    assert_eq!(game.shots,0);assert_eq!(game.objects[ROCKET_0].x,0);
    game.objects[FIGHTER].write_odd_control(0xa1);
    game.launch_rocket();
    assert_eq!(game.shots,1);assert_eq!(game.objects[ROCKET_0].odd_control(),0xa1);
    game.hitd_dspchr_rckt(ROCKET_0,24);
    assert_eq!(game.objects[ROCKET_0].odd_control(),0);
    assert_eq!(game.objects[ROCKET_0].y,0);
}

#[test]
fn original_sprite_copy_and_formation_store_preserve_full_control_byte() {
    let mut game=Game::default();
    game.main_task_enable.fill(0);game.main_task_enable[12]=1;game.counters[0]=1;
    game.objects[4].state=State::Standby;game.formation.coordinates[14]=0xa179;
    game.cpu0_rst38();
    assert_eq!(game.objects[4].y,0x179);
    assert_eq!(game.objects[4].odd_control(),0xa1);
    game.f_05bf();assert_eq!(game.sprite_ram[4].odd_control(),0xa1);
    game.c_sctrl_sprite_ram_clr();assert_eq!(game.objects[4].odd_control(),0);
}

#[test]
fn original_motion_and_bomb_output_replace_only_odd_control_bit_zero() {
    let mut game=Game::default();
    game.objects[FIGHTER].x=100;
    game.objects[52].write_odd_control(0xaa);
    let mut object=Object {state:State::Homing,control_odd_upper:0xf0,..Object::default()};
    let mut motion=Motion {duration:2,y:0x7000,x:0x3000,flags:1,bomb_counter:1,bomb_flags:1,..Motion::default()};
    game.mctl_fltpn_dspchr(&mut motion,&mut object,0).unwrap();
    assert_eq!(object.control_odd_upper,0xf0);
    assert_eq!(game.objects[52].control_odd_upper,0xaa);
    assert_eq!(game.objects[52].y,object.y);
}

#[test]
fn rev_b_capture_copies_the_full_control_byte_and_landing_tests_the_full_byte() {
    let mut game=Game::default();game.combat.capture_boss_id=0x30;
    game.objects[24].state=State::HomeReturn;
    game.objects[FIGHTER].write_odd_control(0xc1);game.objects[FIGHTER].flags=3;
    game.combat.capture_status[4]=1;game.f_19b2();game.game_timers[1]=0;game.f_19b2();
    assert_eq!(game.objects[0].odd_control(),0xc1);assert_eq!(game.objects[0].flags,3);
    game.objects[0].state=State::Rescued;game.objects[0].x=0x80;
    game.objects[0].y=0x129;game.combat.capture_boss_id=0;game.combat.capture_status[1]=2;
    game.f_2000();
    assert_eq!(game.combat.capture_status[1],2); // C1 is not the original comparison's 01
    assert_eq!(game.objects[0].y,0x12a);
}

#[test]
fn enabled_original_formation_tasks_do_not_depend_on_presentation_flags() {
    let mut game=Game::default();game.main_task_enable.fill(0);game.main_task_enable[9]=1;
    game.formation.pulsing=false;game.formation.direction=0;game.counters[0]=0;
    game.formation.coordinates[0]=0xbe00;
    game.cpu0_rst38();
    assert_eq!(game.formation.direction,1);assert_eq!(game.formation.coordinates[0],0xbfff);
    game.main_task_enable[9]=0;game.main_task_enable[10]=1;
    game.active_enemy_count=1;
    game.counters[0]=1;game.formation.sliding=false;
    game.formation.direction=0;game.formation.coordinates[0]=0xabff;
    game.cpu0_rst38();assert_eq!(game.formation.coordinates[0],0xab00);
}

#[test]
fn hardware_wrap_renders_visible_double_height_sprite_even_when_position_byte_is_zero() {
    use galaga_reconstructed::{starfield::Starfield,video};
    let mut objects=[Object::default();64];
    let tiles=[0x24;1024];let colours=[3;1024];let stars=Starfield::default();
    let mut blank=vec![0;video::WIDTH*video::HEIGHT*4];
    video::render(&objects,&tiles,&colours,&stars,0,&mut blank);
    objects[0]=Object {code:6,colour:9,x:0,y:160,flags:8,..Object::default()};
    let mut image=blank.clone();video::render(&objects,&tiles,&colours,&stars,0,&mut image);
    let changed:Vec<_>=image.chunks_exact(4).zip(blank.chunks_exact(4)).enumerate()
        .filter_map(|(pixel,(a,b))|if a!=b {Some(pixel%video::WIDTH)} else {None}).collect();
    assert!(!changed.is_empty());assert!(changed.iter().all(|x|*x<=14));
}
