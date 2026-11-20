//! Original power-on/game recycle, credit task and game-start executive.
//! The original 51xx switches/credit firmware is reconstructed in input51.rs.
use crate::{executive::Background, rom_data::*, Game, ROCKET_0};

/// DAA following ADD/ADC, retaining carry and half-carry from that operation.
pub fn bcd_add(left:u8,right:u8,carry:bool)->(u8,bool) {
    let sum=left as u16+right as u16+u16::from(carry);
    let half=(left&15) as u16+(right&15) as u16+u16::from(carry)>15;
    let mut a=sum as u8;
    let mut correction=0;
    if half || a&15>9 { correction|=6; }
    let cy=sum>255 || a>0x99;
    if cy { correction|=0x60; }
    a=a.wrapping_add(correction);
    (a,cy)
}

/// DAA following SUB, using the subtraction's original borrow flags.
pub fn bcd_sub(left:u8,right:u8)->u8 {
    let mut a=left.wrapping_sub(right);
    if (left&15)<(right&15) { a=a.wrapping_sub(6); }
    if left<right { a=a.wrapping_sub(0x60); }
    a
}

impl Game {
    /// Native post-test entry retained for focused routine tests. Production
    /// browser power-on uses `cold_boot_with_switches` below.
    pub fn power_on()->Self {
        Self::power_on_with_switches(0xf7,0x97)
    }

    pub fn power_on_with_switches(dswa:u8,dswb:u8)->Self {
        let mut g=Self::default();
        g.configure_machine(dswa,dswb);
        g.input51.reset_007e();
        for parameter in g.io_parameters {g.input51.command(parameter);}
        // Entry follows the original self-test's configuration/validity poll.
        // Prime its idle switch history through the recovered firmware loop.
        for _ in 0..4 {g.input51.video_sample_0097([15;4]);}
        g.io_input=g.input51.transfer_three();
        g.j_game_init_02d3();g.render();g
    }

    /// Original cold-start route: the board RAM/ROM result and configuration
    /// display, sound check, crosshatch, controller configuration and finally
    /// main:02D3. The destructive electrical memory checks are represented by
    /// validated Rust storage/imported assets; their original visible success
    /// path and frame waits remain in `service_enter_336c`.
    pub fn cold_boot_with_switches(dswa:u8,dswb:u8)->Self {
        let mut g=Self::default();
        g.configure_machine(dswa,dswb);
        g.service_enter_336c();
        g.render();
        g
    }

    pub(crate) fn j_game_init_02d3(&mut self) {
        self.service_active=false;
        self.game_timers.fill(0);self.combat.sound_requests.fill(0);
        self.reset_player_data();self.stars.reset();
        self.flip_screen=0;self.star_control[0]=0;
        self.boss_sorties.fill(Default::default());
        self.tile_ram[..0x40].fill(0x24);self.tile_ram[0x3c0..].fill(0x24);
        self.colour_ram[..0x40].fill(3);self.c_sctrl_playfld_clr();
        self.initialise_top5();self.game_state=1;
        self.c_sctrl_sprite_ram_clr();
        self.tile_ram[0x3ed..0x3f2].copy_from_slice(&HIGH_SCORE_INITIAL[..5]);
        self.tile_ram[0x3cb..0x3dc].copy_from_slice(&HEADER_TILES);
        self.c_1242_init_taskman_structs();self.motions.fill(Default::default());
        self.main_task_enable[30]=0x20;
        self.credit_count=self.io_input[0];
        self.main_task_enable[30]=0;self.sub_task_enable[0]=0;
        self.g_main();
    }

    /// main:1242..125A, both tables are copied, then only active slot 0 is killed.
    pub fn c_1242_init_taskman_structs(&mut self) {
        self.main_task_enable=MAIN_TASK_DEFAULTS;
        self.reserve_task_enable=MAIN_TASK_DEFAULTS;
        self.main_task_enable[0]=0;
    }

    /// main:035A. Only the 9200..927F globals and notifications are reset here.
    pub fn g_main(&mut self) {
        self.flip_screen=0; self.main_task_enable[18]=0;
        self.combat.hit_notifications.fill(0);
        self.game_state=0; self.attract_state=0; self.attract_text_index=0;self.attract_setup_pending=false;
        self.demo_timer=0; self.demo_target=0; self.enemy_enabled=0;
        self.formation.direction=0; self.formation_signage=0; self.restart_stage=0;
        self.shared_9260.fill(0); self.no_restart_stage=0; self.fighter_collision=0;
        self.star_control[5]=6;
        self.motions.fill(Default::default()); self.c_sctrl_sprite_ram_clr();
        self.c_1242_init_taskman_structs();
        self.game_state=if self.credit_count==0 {1} else {2};
        if self.credit_count==0 {
            self.attract_state=0; self.main_task_enable[2]=1;
            self.background=Background::AttractWait;
        } else { self.l_game_state_ready(); }
    }

    /// main:03A0, original three optional bonus lines and sprite table pointer.
    pub(crate) fn l_game_state_ready(&mut self) {
        self.enemy_enabled=0; self.c_string_out(0x13,None); self.c_string_out(1,None);
        self.sprite_tiles_pointer=0x0452;
        if self.machine_bonus[0]!=0xff {
            self.c_game_bonus_info_show_line(0x1b,self.machine_bonus[0]);
            if self.machine_bonus[1]!=0xff {
                self.c_game_bonus_info_show_line(0x1c,self.machine_bonus[1]&0x7f);
                if self.machine_bonus[1]&0x80==0 {
                    self.c_game_bonus_info_show_line(0x1d,self.machine_bonus[1]&0x7f);
                }
            }
        }
        self.background=Background::ReadyWait;
    }

    /// main:0433. Keep the low-byte +40 and the decimal-output/string calls.
    pub fn c_game_bonus_info_show_line(&mut self,index:u8,digit:u8) {
        let destination=self.c_string_out(index,None);
        let destination=(destination&0xff00)|((destination as u8).wrapping_add(0x40) as u16);
        let destination=self.c_text_out_i_to_d(digit as u16,destination);
        self.c_string_out(0x1e,Some(destination));
        self.c_sprite_tiles_displ();
    }

    /// main:03DF..041C, then yield at the original opening-theme timer wait.
    pub(crate) fn game_start(&mut self) {
        self.combat.sound_requests[0x17]=self.game_state;
        self.c_sctrl_playfld_clr(); self.c_sctrl_sprite_ram_clr();
        self.stars.reset();
        self.reset_player_data(); self.suspended_player.fill(0);
        self.suspended_objects[..16].fill(0);
        self.combat.sound_requests[0x17]=0; self.star_control[0]=0;
        self.combat.sound_requests[0x0b]=1;
        self.main_task_enable[0x12]=1; self.reserve_task_enable[0x12]=1;
        self.gctl_game_init(); self.c_game_or_demo_init(); self.c_string_out(4,None);
        self.game_timers[3]=8; self.background=Background::OpeningThemeWait;
    }

    fn reset_player_data(&mut self) {
        self.active_player_ram.fill(0);
        self.reserve_fighters=0; self.stage_number=0; self.wave_cursor=0;
        self.nest_lr_inhibit=0; self.wave_number=0; self.two_fighters=0;
        self.combat.capture_boss_id=0; self.capture_motion_slot=0;
        self.capture_connected=0; self.combat.capture_boss_flag=0; self.boss_escort_toggle=0;
        self.combat.bonus_bee_id=0; self.bonus_bee_colour=0; self.bonus_bee_class=0;
        self.combat.boss_score_attributes.fill(0); self.next_bonus=0;
        self.player_number=0; self.bonus_bee_timer=0; self.attack_wave_enabled=0;
        self.player_enemy_count=0; self.combat.hits=0; self.shots=0;
    }

    /// main:0466 and gctl_init_puts. Machine stores reserves, excluding fighter up.
    pub fn gctl_game_init(&mut self) {
        self.reserve_fighters=self.machine_reserves;
        self.suspended_reserves=self.machine_reserves; self.suspended_player[0]=self.machine_reserves;
        self.gctl_init_puts(0,0x3f8);
        self.gctl_init_puts(if self.two_player_game {0} else {2},0x3e3);
    }

    /// main:127B, ten missile objects, preserving other bytes in each sprite.
    pub fn c_game_or_demo_init(&mut self) {
        for n in 0..10 {
            let object=&mut self.objects[ROCKET_0+n];
            object.code=0x30; object.x=0; object.flags=if n<2 {0} else {1};
            object.colour=if n<2 {9} else {11};
        }
    }

    /// main:0977..0A26, game-time BCD, credit display and credit-change decisions.
    pub fn f_0977(&mut self) {
        if self.io_input[0]==0xbb {self.service_enter_336c();return;}
        if self.game_state==3 {
            let (mut value,_)=bcd_add(self.game_time_bcd[3],1,false);
            let mut carry=value==0x60;
            if carry {value=0;}
            for i in (0..4).rev() {
                self.game_time_bcd[i]=value;
                if i!=0 { (value,carry)=bcd_add(self.game_time_bcd[i-1],0,carry); }
            }
        } else {
            let mut destination=0x3c;
            if self.credit_count==0xa0 {
                for &tile in CREDIT_TEXT[6..].iter().rev() { self.tile_ram[destination]=tile; destination-=1; }
            } else {
                for &tile in CREDIT_TEXT[..6].iter().rev() { self.tile_ram[destination]=tile; destination-=1; }
                destination-=1;
                let high=self.io_input[0]>>4;
                if high!=0 {self.tile_ram[destination]=high; destination-=1;}
                self.tile_ram[destination]=self.io_input[0]&15;
                self.tile_ram[destination-1]=0x24;
            }
        }
        if self.game_state==0 {return;}
        if self.game_state==1 && self.io_input[0]!=0 {
            self.game_state=2;
            self.combat.sound_requests[..8].fill(0); self.combat.sound_requests[9..24].fill(0);
        }
        let current=self.io_input[0]; let previous=self.credit_count;
        if current==previous {return;}
        if current<previous {
            let debit=bcd_sub(previous,current).wrapping_sub(1);
            self.two_player_game=debit!=0;
            self.credit_count=current; self.game_state=3;
        } else {
            self.credit_count=current;
            if current!=0xa0 {self.coin_sound_count=bcd_sub(current,previous);}
        }
    }

    /// main:06DE, the two original HALTs become separate interrupt continuations.
    pub fn g_halt(&mut self) { self.background=Background::HaltFirstInterrupt; }
    pub(crate) fn g_halt_complete(&mut self) {
        self.c_093c(0); self.combat.sound_requests.fill(0);
        self.c_mach_info_add_score(0xf9); self.c_mach_info_add_score(0xe4);
        let (value,carry)=bcd_add(self.total_plays_bcd[1],u8::from(self.two_player_game)+1,false);
        self.total_plays_bcd[1]=value;
        if carry { self.total_plays_bcd[0]=bcd_add(self.total_plays_bcd[0],1,false).0; }
        self.g_main();
    }

    /// main:0A27, original RRD packing followed by four ADC/DAA additions.
    pub fn c_mach_info_add_score(&mut self,mut score_low:u8) {
        let mut temporary=[0u8;4]; let mut pointer=3;
        for b in (1..=5).rev() {
            let mut a=self.tile_ram[0x300+score_low as usize]; score_low=score_low.wrapping_add(1);
            if a==0x24 {a=0;}
            temporary[pointer]=((a&15)<<4)|(temporary[pointer]>>4);
            if b&1==0 {pointer-=1;}
        }
        temporary[pointer]>>=4; pointer-=1; temporary[pointer]=0;
        let mut carry=false;
        for i in (0..4).rev() { (self.total_score_bcd[i],carry)=bcd_add(self.total_score_bcd[i],temporary[i],carry); }
    }

}
