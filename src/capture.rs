//! Original capture, beam, spin, rescue and captured-fighter tasks.
use crate::{rom_data::TRACTOR_BEAM_TILES, Game, State, FIGHTER};

impl Game {
    /// main:2188. Gray-coded sprite flip flags select spin direction. Returns
    /// the original A and B results, including B=1/A=0 on an upright stop.
    pub fn c_2188_ship_spin(&mut self, id: usize) -> (u8, u8) {
        let object = &mut self.objects[id];
        let mut c = (object.flags >> 1) ^ object.flags;
        let a = object.code & 7;
        if a == 6 && c == 0 && self.combat.capture_status[3] == 0 { return (0, 1); }
        loop {
            if c & 1 == 0 {
                if a != 6 { object.code = object.code.wrapping_add(1); break; }
            } else if a != 0 { object.code = object.code.wrapping_sub(1); break; }
            c = c.wrapping_sub(1);
            if c & 0x80 != 0 { c = 3; }
        }
        let control = if c & 2 != 0 { c ^ 1 } else { c };
        object.flags = control;
        (control, 0)
    }

    /// main:2000..20F1. Original spin, horizontal centring, vertical landing
    /// and the two distinct tails for joining or replacing the active fighter.
    pub fn f_2000(&mut self) {
        let id = self.combat.capture_boss_id as usize / 2;
        if self.objects[id].state != State::Rescued { self.l_20bf(); return; }
        match self.combat.capture_status[1] {
            0 => { self.combat.capture_status[1] = 1; self.game_timers[1] = 2; return; }
            1 => {
                self.combat.capture_status[3] = self.flying_count | self.game_timers[1];
                let (a, b) = self.c_2188_ship_spin(id);
                if b.wrapping_sub(1) != 0 { return; }
                self.main_task_enable[20] = a; self.main_task_enable[21] = a; self.sub_task_enable[5] = a;
                self.combat.capture_status[1] = 2;
                return;
            }
            _ => {}
        }
        let object = &mut self.objects[id];
        if object.x != 0x80 {
            if object.x.wrapping_sub(0x80) & 0x80 == 0 { object.x = object.x.wrapping_sub(1); }
            else { object.x = object.x.wrapping_add(1); }
        } else if self.flip_screen == 0 {
            if object.y as u8 == 0x29 && object.odd_control()==1 { self.combat.capture_status[1] = 3; }
            else { object.y = object.y.wrapping_add(1) & 0x1ff; }
        } else if object.y as u8 == 0x37 && object.odd_control()==0 { self.combat.capture_status[1] = 3; }
        else { object.y = object.y.wrapping_sub(1) & 0x1ff; }
        let c = self.objects[FIGHTER].code.wrapping_sub(6);
        if c == 0 && self.objects[FIGHTER].x != 0x71 {
            let x = &mut self.objects[FIGHTER].x;
            if x.wrapping_sub(0x71) & 0x80 == 0 { *x = x.wrapping_sub(1); }
            else { *x = x.wrapping_add(1); }
            return;
        }
        if self.combat.capture_status[1] != 3 { return; }
        self.objects[id].x = 0;
        let destination = if c == 0 { self.two_fighters = 1; FIGHTER - 1 }
                          else { self.combat.capture_boss_flag = 0; FIGHTER };
        self.objects[destination].y = self.objects[id].y;
        let odd=self.objects[id].odd_control();self.objects[destination].write_odd_control(odd);
        self.objects[id].state = State::Inactive;
        self.objects[destination].code = 6; self.objects[destination].colour = 9;
        self.objects[destination].x = 0x80;
        self.main_task_enable[20] = 1; self.main_task_enable[21] = 1; self.sub_task_enable[5] = 1;
        self.star_control[0] = 1;
        self.l_20bf();
    }

    fn l_20bf(&mut self) { self.main_task_enable[0x1d] = 0; self.combat.sound_requests[0x11] = 0; }

    /// main:20F2..2187, tractor-beam movement of the active fighter.
    pub fn f_20f2(&mut self) {
        let (_, b) = self.c_2188_ship_spin(FIGHTER);
        if b & 1 != 0 {
            if self.main_task_enable[21] == 0 {
                self.capture_connected = 1;
                self.l_217f(); return;
            }
        } else if self.combat.capture_status[1] & 0x80 == 0 {
            if self.combat.capture_status[3] == 0 { return; }
            let boss_x = self.objects[self.combat.capture_boss_id as usize / 2].x;
            let fighter = &mut self.objects[FIGHTER];
            let difference = boss_x.wrapping_sub(fighter.x);
            if difference != 0 {
                if difference & 0x80 == 0 { fighter.x = fighter.x.wrapping_add(1); }
                else { fighter.x = fighter.x.wrapping_sub(1); }
            }
            let row = if self.flip_screen == 0 {
                fighter.y = fighter.y.wrapping_sub(1) & 0x1ff;
                fighter.y as u8
            } else {
                // Original flipped branch increments the low byte only.
                fighter.y = (fighter.y & 0x100) | ((fighter.y as u8).wrapping_add(1) as u16);
                fighter.y as u8
            };
            let disable_at = if self.flip_screen == 0 { 0xe6 } else { 0x7a };
            let connect_at = if self.flip_screen == 0 { 0xe0 } else { 0x80 };
            if row == disable_at { self.main_task_enable[21] = 0; }
            else if row == connect_at { self.combat.capture_status[3] = 0; fighter.colour = 7; }
            return;
        }
        // Boss shot while beaming: move the spinning fighter back down.
        let fighter = &mut self.objects[FIGHTER];
        let target = if self.flip_screen == 0 { 0x29 } else { 0x37 };
        if fighter.y as u8 != target {
            if self.flip_screen == 0 { fighter.y = fighter.y.wrapping_add(1) & 0x1ff; }
            else { fighter.y = (fighter.y & 0x100) | ((fighter.y as u8).wrapping_sub(1) as u16); }
            return;
        }
        if b.wrapping_sub(1) == 0 { self.l_217f(); }
    }

    fn l_217f(&mut self) { self.main_task_enable[0x1c] = 0; self.sub_task_enable[5] = 1; }

    /// main:21CB..2221, wait for capture approach then rotate into beam stance.
    pub fn f_21cb(&mut self) {
        let id = self.combat.capture_boss_id as usize / 2;
        if self.objects[id].state != State::Homing {
            self.main_task_enable[0x19] = 0; self.combat.capture_boss_flag = 0; return;
        }
        let motion = &mut self.motions[self.capture_motion_slot as usize];
        if motion.odd_displacement != 0 { return; }
        motion.rotation = if motion.angle & 0x100 == 0 { 12 } else { -12 };
        if ((motion.angle >> 1) as u8).wrapping_sub(0x78) >= 0x10 { return; }
        self.capture_pause = self.stage_parameters[6];
        motion.rotation = 0;
        self.main_task_enable[0x19] = 0;
        self.combat.capture_status[1] = 0;
        self.capture_connected = 0;
        self.main_task_enable[0x18] = 1;
        self.combat.capture_status[2] = 1; self.combat.capture_status[3] = 1;
    }

    /// main:2222..23A0. Original tile/colour-RAM beam, expansion/erasure,
    /// capture window, boss-hit recovery and captured-ship task transition.
    pub fn f_2222(&mut self) {
        let frame = self.counters[0];
        if frame & 3 == 0 {
            let a = self.combat.capture_status[0].wrapping_neg().wrapping_sub(0x18);
            let mut address = beam_address(a, 0x21, 0x15) & 0x3ff;
            let mut colour = (frame.rotate_right(2)) & 3;
            if colour == 0 { colour += 1; }
            colour += 0x17;
            for _ in 0..6 {
                for _ in 0..10 {
                    self.colour_ram[address as usize] = colour;
                    address = (address & 0x300) | ((address as u8).wrapping_add(1) as u16);
                }
                address = address.wrapping_add(0x16) & 0x3ff;
            }
        }
        if self.combat.capture_status[1] & 0x80 == 0 {
            let id = self.combat.capture_boss_id as usize / 2;
            if self.objects[id].state != State::Homing {
                self.capture_pause = 3;
                self.combat.capture_status[1] = 0x80; self.combat.capture_status[3] = 0;
                self.star_control[1] = 0; self.combat.capture_status[2] = 1;
                self.main_task_enable[20] = 1;
                return;
            }
        }
        self.combat.capture_status[2] = self.combat.capture_status[2].wrapping_sub(1);
        if self.combat.capture_status[2] != 0 { self.l_233d(); return; }
        self.combat.capture_status[2] = self.capture_pause;
        if self.combat.capture_status[1] & 0x80 != 0 {
            self.combat.capture_status[1] = self.combat.capture_status[1].wrapping_add(1);
            let phase = self.combat.capture_status[1] & 15;
            if phase == 11 {
                self.main_task_enable[0x18] = 0;
                self.combat.sound_requests[5] = 0; self.combat.sound_requests[6] = 0;
                self.combat.capture_boss_flag = 0;
            } else { self.beam_row(phase, false); }
            return;
        }
        self.combat.sound_requests[5] = self.capture_pause;
        self.motions[self.capture_motion_slot as usize].duration = 0xff;
        self.combat.capture_status[1] = self.combat.capture_status[1].wrapping_add(1);
        let phase = self.combat.capture_status[1] & 15;
        if phase != 11 {
            if self.combat.capture_status[1] & 0x40 == 0 { self.beam_row(phase, true); }
            else { self.beam_row(phase.wrapping_neg().wrapping_add(11), false); }
            return;
        }
        if self.combat.capture_status[1] & 0x40 == 0 {
            self.combat.capture_status[2] = 0x40; self.combat.capture_status[1] = 0x40;
            return;
        }
        if self.capture_connected != 0 && self.combat.capture_status[1] & 0x20 == 0 {
            self.combat.capture_status[1] = 0x68; return;
        }
        self.main_task_enable[0x18] = 0;
        self.combat.sound_requests[5] = 0; self.combat.sound_requests[6] = 0;
        if self.capture_connected == 0 {
            self.combat.capture_boss_flag = 0;
            self.combat.capture_boss_id = 1;
            self.motions[self.capture_motion_slot as usize].duration = 1;
        } else {
            self.motions[self.capture_motion_slot as usize].path = 0x046b;
            self.star_control[1] = 0;
            self.main_task_enable[0x11] = 1; self.combat.capture_status[4] = 1;
        }
    }

    fn beam_row(&mut self, phase: u8, draw: bool) {
        let mut address = self.c_238a(phase);
        for column in 0..6 {
            self.tile_ram[(address & 0x3ff) as usize] = if draw {
                TRACTOR_BEAM_TILES[(phase as usize - 1) * 6 + column]
            } else { 0x24 };
            address = address.wrapping_sub(0x20);
        }
    }

    /// main:238A, return the original DE tile address.
    fn c_238a(&self, phase: u8) -> u16 {
        beam_address(self.combat.capture_status[0].wrapping_neg().wrapping_add(0x10), 0x20, 0x14)
            .wrapping_add(phase as u16)
    }

    /// main:233D, test the original unsigned beam-width rectangle and flags.
    fn l_233d(&mut self) {
        if self.combat.capture_status[1] != 0x40 { return; }
        let mut x = self.objects[FIGHTER].x;
        if self.flip_screen & 1 != 0 { x = x.wrapping_add(14).wrapping_neg(); }
        if self.combat.capture_status[0].wrapping_sub(x).wrapping_add(0x1b) >= 0x36 { return; }
        if self.game_state != 1 && (self.restart_stage ^ 1) & self.main_task_enable[20] == 0 { return; }
        self.main_task_enable[20] = 0; self.combat.sound_requests[5] = 0;
        self.sub_task_enable[5] = 0; self.restart_stage = 0;
        self.main_task_enable[0x1c] = 1; self.combat.sound_requests[6] = 1; self.star_control[1] = 1;
        self.capture_pause = 10;
    }

    /// main:19B2..1A85. Rev. B delays the transfer into the captured-object
    /// banks until the text timer expires, and copies both control bytes.
    pub fn f_19b2(&mut self) {
        let boss = self.combat.capture_boss_id as usize / 2;
        let captured = (self.combat.capture_boss_id & 7) as usize / 2;
        if self.combat.capture_status[4] != 0 {
            self.c_string_out(10, None);
            self.game_timers[1] = 6;
            self.objects[FIGHTER].colour = 7;
            self.combat.capture_status[1] = 0; self.combat.capture_status[4] = 0;
            return;
        }
        if self.game_timers[1] != 0 {
            if self.game_timers[1] == 4 {
                self.game_timers[1] = 3; self.combat.sound_requests[9] = 3;
            }
            self.motions[self.capture_motion_slot as usize].duration = 4;
            return;
        }
        if self.tile_ram[0x2d1] != 0x24 {
            self.objects[captured].x = self.objects[FIGHTER].x;
            self.objects[FIGHTER].x = 0;
            self.objects[captured].y = self.objects[FIGHTER].y;
            let odd=self.objects[FIGHTER].odd_control();self.objects[captured].write_odd_control(odd);
            self.objects[captured].flags = self.objects[FIGHTER].flags;
            self.objects[captured].code = 7; self.objects[captured].colour = 7;
            self.c_string_out(11, Some(0x83b1));
        }
        if self.objects[boss].state == State::Homing {
            self.objects[captured].x = self.objects[boss].x;
            self.objects[captured].y = self.objects[boss].y
                .wrapping_add(if self.flip_screen & 1 == 0 { 16 } else { (-16i16) as u16 }) & 0x1ff;
            self.objects[captured].control_odd_upper=self.objects[boss].control_odd_upper;
            return;
        }
        let previous = self.combat.capture_status[1];
        if previous == 0 { self.objects[captured].code = 6; }
        self.combat.capture_status[1] = previous.wrapping_add(1);
        if previous != 0x24 {
            self.objects[captured].y = self.objects[captured].y
                .wrapping_add(if self.flip_screen & 1 == 0 { u16::MAX } else { 1 }) & 0x1ff;
            return;
        }
        self.main_task_enable[0x11] = 0; self.combat.sound_requests[9] = 0;
        self.objects[captured].state = State::Standby;
        self.combat.capture_boss_id = 1; self.star_control[0] = 1;
        self.restart_stage = 2;
    }
}

fn beam_address(mut a: u8, mut high: u8, low: u8) -> u16 {
    for _ in 0..2 {
        let carry = a >> 7;
        a = a.rotate_left(1);
        high = (high << 1) | carry;
    }
    u16::from_le_bytes([(a & 0xe0).wrapping_add(low), high])
}
