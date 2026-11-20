//! Original 1700..19B1 scripted fighter and attract-mode state progression.
use crate::{rom_data::*, Game, State, FIGHTER};

fn byte(address:u16)->u8 {
    match address {
        0x0452..=0x045d=>BONUS_SHIP_TILES[(address-0x0452) as usize],
        0x181f..=0x183f=>DEMO_POST_CAPTURE[(address-0x181f) as usize],
        0x1887..=0x18ab=>DEMO_COMBAT[(address-0x1887) as usize],
        0x1928..=0x193f=>DEMO_TRAINING[(address-0x1928) as usize],
        0x195c..=0x1983=>ATTRACT_SPRITES[(address-0x195c) as usize],
        _=>panic!("Unmapped original attract data {address:04x}"),
    }
}

impl Game {
    /// main:129E, packed colour/code and nine-bit Y, retaining CTRL even byte.
    pub fn c_sprite_tiles_displ(&mut self) {
        let pointer=self.sprite_tiles_pointer;
        let object=&mut self.objects[byte(pointer) as usize/2];
        let packed=byte(pointer+1);
        object.code=(packed&0x78).wrapping_add(6);
        object.colour=(packed&7)|((packed>>4)&8);
        object.state=State::Standby;
        object.x=byte(pointer+2); object.y=(byte(pointer+3) as u16)*2;
        object.write_odd_control((object.y>>8) as u8);
        self.sprite_tiles_pointer=pointer+4;
    }

    /// main:1700, first switch controls the CURRENT vector token's action.
    pub fn f_1700(&mut self) {
        let token=byte(self.demo_vector);
        match token.rotate_left(3)&7 {
            2=> {
                if self.counters[0]&15!=0 {return;}
                self.demo_timer=self.demo_timer.wrapping_sub(1);
                if self.demo_timer!=0 {return;}
            }
            4|5=> {
                if token.rotate_left(3)&7==5 {self.launch_rocket();}
                let controls=if token&1==0 {token&10} else {
                    let x=self.objects[FIGHTER].x;
                    let target=self.objects[self.demo_target as usize/2].x;
                    if x==target {10} else if x<target {8} else {2}
                };
                self.fighter_control(controls);
                if self.counters[0]&3!=0 {return;}
                self.demo_timer=self.demo_timer.wrapping_sub(1);
                if self.demo_timer!=0 {return;}
                self.launch_rocket();
            }
            0|1|3=>{},
            _=>{ self.fault=Some(0x1713); return; }
        }
        self.case_1766();
    }

    /// main:1766, advance the vector and dispatch its NEXT token's parameter.
    fn case_1766(&mut self) {
        if byte(self.demo_vector)&0xc0==0x80 {self.demo_vector+=1;}
        self.demo_vector+=1;
        let token=byte(self.demo_vector);
        match token.rotate_left(3)&7 {
            0|1=>self.demo_target=token.rotate_left(1)&0x7e,
            2=>self.demo_timer=token&0x1f,
            3=>{self.c_string_out(token&0x1f,None);}
            4|5=>self.demo_timer=byte(self.demo_vector+1),
            6=>self.main_task_enable[3]=0,
            _=>self.fault=Some(0x1786),
        }
    }

    fn l_attmode_state_step(&mut self) {
        self.attract_state=self.attract_state.wrapping_add(1);
        if self.attract_state==15 {self.attract_state=0;}
    }

    /// main:17B2, all fifteen original table entries, including the training
    /// sequence, first-stage demo, captured-fighter sequence and heroes screen.
    pub fn f_17b2(&mut self) {
        if self.game_state!=1 {return;}
        match self.attract_state {
            0|6|13=> {self.c_sctrl_playfld_clr(); self.c_sctrl_sprite_ram_clr();}
            1=> {
                self.sprite_tiles_pointer=0x195c; self.attract_text_index=0;
                self.combat.flight_hits_remaining=0; self.game_timers[2]=2;
            }
            2=> {
                if self.game_timers[2]!=0 {return;}
                self.game_timers[2]=2;
                if self.attract_text_index!=5 {
                    self.attract_text_index=self.attract_text_index.wrapping_add(1);
                    self.c_string_out(0x0d+self.attract_text_index,None);
                    if self.attract_text_index>=3 {self.c_sprite_tiles_displ();}
                    return;
                }
            }
            3=> {
                if !self.attract_setup_pending {
                    for _ in 0..7 {self.c_sprite_tiles_displ();}
                    self.reserve_fighters=0;self.main_task_enable[5]=0;
                    self.attract_setup_pending=true;
                }
                // c_134C waits in this IRQ while the independent sub board
                // finishes flying objects. Resume the same call/task below.
                if self.flying_count!=0 {return;}
                self.attract_setup_pending=false;
                self.c_133a_fghtr_onscreen();
                for (index,value) in [(5,0x0d),(6,0xff),(4,0x0d),(5,0xff),(1,0x0d),(2,0xff),(0,0x0d),(1,0xff)] {
                    self.bomber_control[index]=value;
                }
                self.demo_vector=0x1928;
                for sortie in &mut self.boss_sorties {sortie.object_id=0;sortie.path=0;}
                self.two_fighters=0; self.enemy_enabled=0; self.combat.capture_boss_flag=1;
                self.main_task_enable[16]=1; self.main_task_enable[11]=1; self.main_task_enable[3]=1;
                self.combat.sound_requests[23]=self.attract_sound_dsw.rotate_right(1)&1;
                self.c_game_or_demo_init();
            }
            4|9|11=> {if self.main_task_enable[3]!=0 {return;}}
            5=> {
                match self.game_timers[2] {
                    0=> {self.combat.hit_notifications[0x34/2]=0x34; self.game_timers[2]=9; return;}
                    1=>{},
                    6=> {
                        self.objects[FIGHTER].x=0; self.c_string_out(0x13,None); self.c_string_out(0x14,None); return;
                    }
                    _=>return,
                }
            }
            7=> {
                if self.counters[0]&31!=31 {return;}
                self.main_task_enable[5]=1; self.c_string_out(2,None);
            }
            8=> {
                self.combat.capture_boss_flag=0; self.combat.sound_requests[23]=1;
                self.stage_number=1; self.main_task_enable[3]=1; self.main_task_enable[21]=1;
                self.demo_vector=0x1887; self.stg_init_env();
                if self.fault.is_some() {return;}
                self.c_133a_fghtr_onscreen();
                self.enemy_enabled=1; self.attack_wave_enabled=1; self.boss_escort_toggle=1;
                self.stage_parameters[4]=2; self.stage_parameters[5]=2;
            }
            10=> {
                if self.flying_count!=0 {return;}
                self.c_133a_fghtr_onscreen(); self.demo_vector=0x181f;
                self.main_task_enable[3]=1; self.main_task_enable[21]=1; self.sub_task_enable[5]=1;
            }
            12=> {
                self.motions.fill(Default::default()); self.c_1242_init_taskman_structs();
                self.main_task_enable[16]=0; self.enemy_enabled=0; self.main_task_enable[2]=1;
            }
            14=> {
                match self.game_timers[3] {
                    0=> {self.c_mach_hiscore_show(); self.game_timers[3]=10;return;}
                    1=>{},
                    _=>return,
                }
            }
            _=>{self.fault=Some(0x17c3);return;}
        }
        self.l_attmode_state_step();
    }

    /// main:1D7C, original star-control acceleration/residual routine. Raster
    /// star generation itself belongs to the browser hardware adapter.
    pub fn f_1d76(&mut self) {
        if self.star_control[0]==0 {
            self.star_control[1]=0; self.star_control[3]=0; self.star_control[4]=0;
            self.star_control[5]=7; return;
        }
        let mut a=0xfd;
        if self.star_control[1]==0 {
            if self.star_control[2]!=self.star_control[3] {self.star_control[3]=self.star_control[3].wrapping_add(1);}
            let sum=self.star_control[3].wrapping_add(self.star_control[4]);
            self.star_control[4]=sum&0x3f; a=sum.rotate_left(2)&3;
        }
        if self.flip_screen&1==0 {a=a.wrapping_neg();}
        self.star_control[5]=a.wrapping_sub(1)&7;
    }
}
