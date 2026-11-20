//! Original 110C player-state/object/task exchange and 058E changeover.
use crate::{executive::Background, Game, State};

impl Game {
    /// The original 64-byte player record, retaining its unused bytes too.
    /// Named Rust fields remain direct ports of the record's original members.
    fn player_record(&self)->[u8;64] {
        let mut p=self.active_player_ram;
        p[0]=self.reserve_fighters;p[1]=self.stage_number;
        p[2..4].copy_from_slice(&0x8920u16.wrapping_add(self.wave_cursor as u16).to_le_bytes());
        p[4]=self.nest_lr_inhibit;p[5]=self.stage_number.wrapping_add(1)&3;
        p[6]=self.wave_number;p[7]=self.two_fighters;
        p[8]=self.combat.capture_boss_id;p[9]=self.capture_motion_slot*20;
        p[10]=self.capture_connected;p[11]=self.combat.capture_boss_flag;
        p[12]=self.boss_escort_toggle;p[13]=self.combat.bonus_bee_id;
        p[14]=self.bonus_bee_class;p[15]=self.bonus_bee_colour;
        p[16..24].copy_from_slice(&self.combat.boss_score_attributes);
        p[30]=self.next_bonus;p[32]=self.player_number;p[33]=self.bonus_bee_timer;
        p[34]=self.attack_wave_enabled;p[35]=self.player_enemy_count;
        p[36..38].copy_from_slice(&self.combat.hits.to_le_bytes());
        p[38..40].copy_from_slice(&self.shots.to_le_bytes());
        p
    }

    fn restore_player_record(&mut self,p:[u8;64]) {
        self.active_player_ram=p;
        self.reserve_fighters=p[0];self.stage_number=p[1];
        self.wave_cursor=u16::from_le_bytes([p[2],p[3]]).wrapping_sub(0x8920) as usize;
        self.nest_lr_inhibit=p[4];self.wave_number=p[6];self.two_fighters=p[7];
        self.combat.capture_boss_id=p[8];self.capture_motion_slot=p[9]/20;
        self.capture_connected=p[10];self.combat.capture_boss_flag=p[11];
        self.boss_escort_toggle=p[12];self.combat.bonus_bee_id=p[13];
        self.bonus_bee_class=p[14];self.bonus_bee_colour=p[15];
        self.combat.boss_score_attributes.copy_from_slice(&p[16..24]);
        self.next_bonus=p[30];self.player_number=p[32];self.bonus_bee_timer=p[33];
        self.attack_wave_enabled=p[34];self.player_enemy_count=p[35];
        self.combat.hits=u16::from_le_bytes([p[36],p[37]]);
        self.shots=u16::from_le_bytes([p[38],p[39]]);
    }

    /// main:110C..117E. Forty-eight packed enemy bytes are exchanged, not
    /// an invented full copy of sprites or coordinates. Only resting enemies
    /// receive the original packed presence/code/colour byte.
    pub fn c_player_active_switch(&mut self) {
        self.main_task_enable[0]=0x1f;self.reserve_task_enable[0]=0x1f;
        let previous=self.player_record();
        self.restore_player_record(self.suspended_player);
        self.suspended_player=previous; self.suspended_reserves=previous[0];
        for i in 0..48 {
            let object=&mut self.objects[i];
            let outgoing=if object.state==State::Standby {
                (object.code&0x78)|(object.colour&7)|0x80
            } else {object.code&0x7f};
            let incoming=self.suspended_objects[i];self.suspended_objects[i]=outgoing;
            if incoming&0x80!=0 {
                object.code=(incoming&0x78).wrapping_add(6);
                object.colour=incoming&7;object.state=State::Standby;
            } else {
                object.code=incoming;object.x=0;object.state=State::Inactive;
            }
        }
        std::mem::swap(&mut self.main_task_enable,&mut self.reserve_task_enable);
        self.main_task_enable[0]=0;
        // Presentation metadata follows the original enables. It does not gate
        // either formation routine when that routine is dispatched.
        self.formation.sliding=self.main_task_enable[10]!=0;
        self.formation.pulsing=self.main_task_enable[9]!=0;
    }

    /// main:05A7..05E8, invoked only once the retreat task has finished.
    pub(crate) fn player_change_exchange(&mut self) {
        self.active_player_ram[40]=self.combat.sound_requests[0];
        self.active_player_ram[31]=self.game_timers[2];
        self.c_player_active_switch();self.c_2c00();
        self.game_timers[2]=self.active_player_ram[31];
        self.combat.sound_requests[0]=self.active_player_ram[40];
        self.draw_resv_ships();
        if self.player_enemy_count!=0 {self.gctl_stg_new_atk_wavs_init();}
        self.flip_screen=self.machine_cabinet&self.player_number;
        self.formation.initialise(0x3f,self.flip_screen);
        self.c_new_level_tokens(0,true);
        self.background=Background::PlayerChangeTokens;
    }

    /// main:1D38..1D7B. The 7E count, direction bit, six nine-bit rows and
    /// two forced MOB updates are retained in the original call order.
    pub fn f_1d32(&mut self) {
        if self.formation_change_timer&0x7f==0x7e {self.main_task_enable[14]=0;return;}
        let previous=self.formation_change_timer;
        self.formation_change_timer=self.formation_change_timer.wrapping_add(1);
        let positive=(self.flip_screen^(previous.rotate_left(1)))&1!=0;
        for row in &mut self.formation.coordinates[10..16] {
            let low=*row as u8;
            let next=if positive {low.wrapping_add(1)} else {low.wrapping_sub(1)};
            let carry=if positive {low==255} else {low==0};
            *row=(*row&0xff00)|next as u16;
            if carry {*row^=0x100;}
        }
        let frame=(self.counters[0]&0xfc).wrapping_add(1);
        self.c_23e0(frame);self.c_23e0(frame.wrapping_add(2));
    }
}
