//! Collision and object-hit routines translated from sub:076A..08D2 and
//! main:1DB9,24B2..2534. The assembly is authoritative where the C aid differs.
use crate::{Game, State, FIGHTER};

#[derive(Clone, Debug)]
pub struct Combat {
    pub hit_notifications: [u8; 48],
    pub hit_multipliers: [u8; 16],
    pub sound_requests: [u8; 32],
    pub hits: u16,
    pub flying_hits: u8,
    /// Despite its 'w_' annotation name, the original hit routine decrements
    /// only the LOW BYTE with DEC (HL), not a 16-bit word.
    pub flight_hits_remaining: u8,
    pub challenge_attributes: [u8; 2],
    pub capture_boss_id: u8,
    pub capture_boss_flag: u8,
    pub capture_status: [u8; 5],
    pub boss_score_attributes: [u8; 8],
    pub bonus_bee_id: u8,
    pub squad_remaining: u8,
    pub squad_bonus_increment: u8,
    pub squad_bonus_sprite: u8,
}

impl Default for Combat {
    fn default() -> Self {
        Self {
            hit_notifications: [0; 48], hit_multipliers: [0; 16],
            sound_requests: [0; 32], hits: 0, flying_hits: 0,
            flight_hits_remaining: 0, challenge_attributes: [0; 2],
            capture_boss_id: 1, capture_boss_flag: 0, capture_status: [0; 5],
            boss_score_attributes: [1, 0xb5, 1, 0xb5, 1, 0xb5, 1, 0xb5],
            bonus_bee_id: 1, squad_remaining: 0, squad_bonus_increment: 0,
            squad_bonus_sprite: 0,
        }
    }
}

impl Game {
    /// sub:076A hitd_det_rckt. Carry tests have deliberately asymmetric
    /// boundaries. Preserve the original scan order and saved rocket position.
    pub fn hitd_det_rckt(&mut self, rocket: usize) {
        let x = self.objects[rocket].x;
        let y = (self.objects[rocket].y >> 1) as u8;
        let start = if self.main_task_enable[0x1d] != 0 { 4 } else { 0 };
        self.hitd_det_rckt_range(rocket, true, x, y, start, 48);
    }

    /// Shared scan entry reached by the original JP l_07B4 after a fighter
    /// hits a bug. The impactor offset stays EVEN in that unusual entry, so
    /// later common calls still clear fighter X, although they count as hits.
    fn hitd_det_rckt_range(&mut self, impactor: usize, clear_y: bool, x: u8, y: u8, start: usize, end: usize) {
        for target in start..end {
            let object = self.objects[target];
            if (object.state as u8 | self.hit_notification_byte(target)) & 0x80 != 0 { continue; }
            if object.state as u8 & 0xfe == 4 { continue; }
            let dy = ((object.y >> 1) as u8).wrapping_sub(y).wrapping_sub(3);
            if !dy.overflowing_add(6).1 { continue; }
            let dx = object.x.wrapping_sub(x);
            let horizontal_hit = if self.two_fighters == 0 {
                dx.wrapping_sub(6).overflowing_add(11).1
            } else {
                let (a, carry) = dx.wrapping_sub(20).overflowing_add(11);
                if carry { true } else {
                    let (a, carry) = a.overflowing_add(4);
                    !carry && a.overflowing_add(11).1
                }
            };
            if horizontal_hit {
                self.combat.hits = self.combat.hits.wrapping_add(1);
                self.clear_hit_impactor(impactor, clear_y);
                let disposition = (object.state as u8).wrapping_sub(1) & 0xfe;
                if !self.hitd_dspchr(target, disposition) { return; }
            }
        }
    }

    fn clear_hit_impactor(&mut self, impactor: usize, clear_y: bool) {
        if clear_y { self.objects[impactor].y = 0;self.objects[impactor].write_odd_control(0); }
        else { self.objects[impactor].x = 0; self.objects[impactor].flags = 0; }
    }

    /// sub:07B9. Clearing the odd sprite register clears Y, not X. The scan
    /// continues at its saved position, just as the original jump does.
    pub fn hitd_dspchr_rckt(&mut self, rocket: usize, target: usize) -> bool {
        self.combat.hits = self.combat.hits.wrapping_add(1);
        self.objects[rocket].y = 0;
        self.objects[rocket].write_odd_control(0);
        let disposition = (self.objects[target].state as u8).wrapping_sub(1) & 0xfe;
        self.hitd_dspchr(target, disposition)
    }

    /// sub:07C2 hitd_dspchr. Returns whether the original caller continues
    /// scanning. All original notification, score and sound-request writes
    /// are retained for the original audio executive's subsequent NMI.
    pub fn hitd_dspchr(&mut self, target: usize, disposition: u8) -> bool {
        let colour = self.objects[target].colour;
        if colour == 0 {
            self.objects[target].colour = 1;
            self.combat.sound_requests[4] = 1;
            return true;
        }
        if colour == 0x0b {
            self.objects[target].x = 0;
            self.objects[target].state = State::Inactive;
            return false;
        }
        let id = (target * 2) as u8;
        if disposition == 0 {
            self.combat.hit_notifications[target] = 0x81;
        } else {
            let slot = self.objects[target].motion_slot as usize;
            if slot >= self.motions.len() { self.fault = Some(0x081e); return false; }
            self.motions[slot].flags = 0;
            self.combat.flying_hits = self.combat.flying_hits.wrapping_add(1);
            self.combat.flight_hits_remaining = self.combat.flight_hits_remaining.wrapping_sub(1);
            if self.combat.flight_hits_remaining == 0 {
                self.combat.hit_notifications[target] = self.combat.challenge_attributes[1];
                self.combat.hit_multipliers[15] = self.combat.hit_multipliers[15]
                    .wrapping_add(self.combat.challenge_attributes[0]);
            } else if colour == 7 {
                self.combat.hit_notifications[target] = 0xb8;
            } else if self.combat.bonus_bee_id == id || id & 0x38 == 0x38 {
                self.combat.squad_remaining = self.combat.squad_remaining.wrapping_sub(1);
                if self.combat.squad_remaining != 0 {
                    self.combat.hit_notifications[target] = 0x81;
                } else {
                    self.combat.hit_multipliers[15] = self.combat.hit_multipliers[15]
                        .wrapping_add(self.combat.squad_bonus_increment);
                    self.combat.hit_notifications[target] = self.combat.squad_bonus_sprite;
                }
            } else if colour != 1 {
                self.combat.hit_notifications[target] = 0x81;
            } else {
                // sub:086A..0898: release a captured fighter if it is diving.
                let captured = (id & 7) as usize / 2;
                if self.objects[captured].state == State::Homing {
                    let slot = self.objects[captured].motion_slot as usize;
                    if slot >= self.motions.len() { self.fault = Some(0x086a); return false; }
                    self.motions[slot].flags = 0;
                    self.objects[captured].colour = 9;
                    self.combat.capture_boss_id = (captured * 2) as u8;
                    self.objects[captured].state = State::Rescued;
                    self.combat.capture_status[1] = 0;
                    self.main_task_enable[0x1d] = 1;
                    self.combat.capture_status[3] = 1;
                    self.combat.sound_requests[0x11] = 1;
                }
                // sub:0899..08B5. The old C translation omits the attribute
                // loads and accidentally uses a motion-slot index as points.
                self.game_timers[1] = 6;
                let attribute = (id & 7) as usize;
                self.combat.hit_multipliers[15] = self.combat.hit_multipliers[15]
                    .wrapping_add(self.combat.boss_score_attributes[attribute]);
                self.combat.hit_notifications[target] = self.combat.boss_score_attributes[attribute + 1];
            }
        }
        // sub:07DF..0814, the shared tail reached by stationary and flying hits.
        if self.combat.capture_boss_id == id {
            self.combat.capture_boss_flag = 0;
            self.combat.capture_boss_id = 1;
        }
        let sound = if colour == 7 { 6 } else { colour.wrapping_sub(1) & 3 };
        self.combat.sound_requests[1 + sound as usize] = 1;
        if colour == 7 { self.combat.capture_boss_flag = 0; }
        self.combat.hit_multipliers[colour as usize] = self.combat.hit_multipliers[colour as usize].wrapping_add(1);
        if disposition != 0 {
            self.combat.hit_multipliers[colour as usize] = self.combat.hit_multipliers[colour as usize].wrapping_add(1);
        }
        true
    }

    /// main:1DB9 (f_1DB3 in the annotation), task slot 0x0B.
    pub fn f_1db3(&mut self) {
        for target in 0..48 {
            if self.combat.hit_notifications[target] & 0x80 == 0 { continue; }
            self.combat.hit_notifications[target] &= 0x7f;
            self.objects[target].state = State::Exploding;
            self.objects[target].motion_slot = 0x40;
            self.objects[target].colour = 0x0a;
        }
    }

    /// main:24B2..2534, reached from the original object-state jump table.
    pub fn case_24b2(&mut self, target: usize) {
        let object = &mut self.objects[target];
        let mut a = object.motion_slot;
        if a != 0x45 {
            a = a.wrapping_add(1);
            object.motion_slot = a;
            if a == 0x45 { a = a.wrapping_add(3); }
            if a == 0x44 {
                object.x = object.x.wrapping_sub(8);
                object.y = object.y.wrapping_sub(8) & 0x1ff;
                object.flags = 0x0c;
            }
            object.code = a;
            return;
        }
        a = self.combat.hit_notifications[target];
        if a == 1 {
            object.x = 0;
            object.flags = 0;
            object.state = State::Inactive;
            return;
        }
        object.code = a;
        if a >= 0x37 { object.colour = if a >= 0x3a { 14 } else { 13 }; }
        object.flags = if a < 0x3b {
            object.x = object.x.wrapping_add(8);
            0
        } else { 8 };
        object.y = object.y.wrapping_add(8) & 0x1ff;
        object.state = State::Score;
        object.motion_slot = 0x13;
    }

    /// main:2535..254C. Counter expires at the object task's interleaved rate.
    pub fn case_2535(&mut self, target: usize) {
        let object = &mut self.objects[target];
        object.motion_slot = object.motion_slot.wrapping_sub(1);
        if object.motion_slot == 0 {
            object.state = State::Inactive;
            object.x = 0;
            object.flags = 0;
        }
    }

    /// main:243C..245E. Fighter explosion (same state is also used for a rogue
    /// fighter). The object dispatcher invokes it on the original cadence.
    pub fn case_243c(&mut self, target: usize) {
        let object = &mut self.objects[target];
        let a = object.motion_slot.wrapping_sub(1);
        if a != 0 {
            object.motion_slot = a;
            if a & 3 == 0 { object.code = object.code.wrapping_add(4); }
        } else {
            object.x = 0;
            object.flags = 0;
            object.state = State::Inactive;
        }
    }

    /// The detector reads the original EVEN byte at 9200+object ID. Bomb
    /// reads at 9268..9276 address unused/service RAM, rather than a fabricated
    /// extension of the 48 alien notification entries. That RAM is cleared by
    /// g_main and is retained as shared bytes for service-mode aliasing.
    pub fn hit_notification_byte(&self, target: usize) -> u8 {
        if target < 48 { self.combat.hit_notifications[target] }
        else { self.shared_9260[(target * 2) - 0x60] }
    }

    /// sub:05EE, including both joined-fighter branches and the original
    /// sprite-buffer/SFR distinction when constructing the explosion.
    pub fn f_05ee(&mut self) {
        let enabled = self.main_task_enable[0x14];
        if enabled == 0 { return; }
        self.no_restart_stage = enabled;
        if self.two_fighters != 0 && self.objects[FIGHTER - 1].x != 0 {
            if self.hitd_fghtr_notif(FIGHTER - 1) {
                self.hitd_fghtr_hit(FIGHTER - 1, self.sprite_ram[FIGHTER - 1].x);
                self.combat.capture_boss_flag = 0;
            }
        }
        if self.objects[FIGHTER].x == 0 { return; }
        if !self.hitd_fghtr_notif(FIGHTER) { return; }
        if self.two_fighters != 0 {
            self.combat.capture_boss_flag = 0;
            self.objects[FIGHTER].x = self.objects[FIGHTER - 1].x;
            // The original tail writes into SFR banks on this branch.
            self.hitd_fghtr_hit_sfr(self.sprite_ram[FIGHTER].x);
            return;
        }
        self.main_task_enable[0x14] = 0;
        self.main_task_enable[0x15] = 0;
        self.sub_task_enable[5] = 0;
        self.star_control[0] = 0;
        self.no_restart_stage = 0;
        self.hitd_fghtr_hit(FIGHTER, self.sprite_ram[FIGHTER].x);
    }

    /// sub:0648 -> 064F. Only the Y LOW BYTE is decremented by eight;
    /// there is deliberately no borrow into the ninth-bit sprite register.
    fn hitd_fghtr_hit(&mut self, fighter: usize, displayed_x: u8) {
        let object = &mut self.objects[fighter];
        object.x = displayed_x.wrapping_sub(8);
        object.y = (object.y & 0x100) | ((object.y as u8).wrapping_sub(8) as u16);
        object.colour = 11; object.code = 0x20; object.flags = 12;
        object.state = State::Rogue; object.motion_slot = 15;
        self.hitd_fghtr_hit_tail();
    }

    fn hitd_fghtr_hit_sfr(&mut self, displayed_x: u8) {
        let object = &mut self.sprite_ram[FIGHTER - 1];
        object.x = displayed_x.wrapping_sub(8);
        object.y = (object.y & 0x100) | ((object.y as u8).wrapping_sub(8) as u16);
        object.colour = 11; object.code = 0x20; object.flags = 12;
        self.objects[FIGHTER - 1].state = State::Rogue;
        self.objects[FIGHTER - 1].motion_slot = 15;
        self.hitd_fghtr_hit_tail();
    }

    fn hitd_fghtr_hit_tail(&mut self) {
        self.two_fighters = 0;
        self.combat.sound_requests[0x19] = self.game_state.wrapping_sub(1);
        if self.no_restart_stage == 0 { self.restart_stage = 1; }
    }

    /// sub:0681..06B6: save position, scan aliens, then always scan bombs.
    pub fn hitd_fghtr_notif(&mut self, fighter: usize) -> bool {
        self.fighter_collision = 0;
        let object = self.objects[fighter];
        if object.state == State::Rogue { return false; }
        let x = object.x;
        let y = (object.y >> 1) as u8;
        let (start, end) = if self.main_task_enable[8] != 0 { (28, 32) } else { (0, 48) };
        self.hitd_det_fghtr(fighter, x, y, start, end);
        self.hitd_det_fghtr(fighter, x, y, 52, 60);
        self.fighter_collision != 0
    }

    /// sub:06B7..06F4. A fatal alien hit continues at the ROCKET detector's
    /// next-object label; a bomb hit returns directly to the caller.
    fn hitd_det_fghtr(&mut self, fighter: usize, x: u8, y: u8, start: usize, end: usize) {
        for target in start..end {
            let object = self.objects[target];
            if (object.state as u8 | self.hit_notification_byte(target)) & 0x80 != 0 { continue; }
            if object.state as u8 & 0xfe == 4 || object.x == 0 { continue; }
            let dx = object.x.wrapping_sub(x).wrapping_sub(7);
            if !dx.overflowing_add(13).1 { continue; }
            let dy = ((object.y >> 1) as u8).wrapping_sub(y).wrapping_sub(4);
            if !dy.overflowing_add(7).1 { continue; }
            self.fighter_collision = 1;
            self.clear_hit_impactor(fighter, false);
            if self.hitd_dspchr(target, 1) {
                self.hitd_det_rckt_range(fighter, false, x, y, target + 1, end);
            }
            return;
        }
    }
}
