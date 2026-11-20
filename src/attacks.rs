//! Original sortie manager main:1B6B..1D37 and launch helpers 1079..110B.
//! Names follow the older annotations; addresses are recorded for rev. B.
use crate::{rom_data::*, Game, State};

#[derive(Clone, Copy, Debug)]
pub struct Sortie { pub object_id: u8, pub path: u16 }
impl Default for Sortie {
    fn default() -> Self { Self { object_id: 0xff, path: 0 } }
}

impl Game {
    /// main:1B6B. Pending boss/escort elements precede the divided-clock
    /// launch timers. A queue entry is consumed even if its alien was hit.
    pub fn f_1b65(&mut self) {
        if self.enemy_enabled != 0
            && (!self.main_task_enable[0x1d] & self.main_task_enable[0x15]) == 0 { return; }
        for queue in 0..4 {
            let sortie = self.boss_sorties[queue];
            if sortie.object_id == 0xff { continue; }
            self.boss_sorties[queue].object_id = 0xff;
            let id = sortie.object_id & 0x7f;
            if self.objects[id as usize / 2].state != State::Standby { return; }
            self.c_1079(sortie.object_id, sortie.path);
            self.combat.sound_requests[0x13] = 1;
            return;
        }
        if self.counters[0] & 15 != 0 { return; }
        let mut timer = 0usize;
        let mut b = 3u8;
        while b != 0 {
            self.bomber_control[timer] = self.bomber_control[timer].wrapping_sub(1);
            if self.bomber_control[timer] == 0 { break; }
            timer += 1;
            b -= 1;
        }
        if b == 0 { return; }
        if self.flying_count >= self.stage_parameters[4] {
            self.bomber_control[timer] = self.bomber_control[timer].wrapping_add(1);
            return;
        }
        self.bomber_control[timer] = self.bomber_control[timer + 4];
        match b - 1 {
            0 => self.case_bmbr_drone(20, 8, 0x034f),
            1 => self.case_bmbr_drone(16, 0x40, 0x03a9),
            2 => self.case_bmbr_boss(),
            _ => unreachable!(),
        }
    }

    /// main:1BDD,1BFD -> 1BE5, common bee/moth scan. The bonus-bee exclusion
    /// and the actual byte written to sound RAM are absent from the old C aid.
    fn case_bmbr_drone(&mut self, mut count: u8, mut id: u8, path: u16) {
        let bonus_bee = self.combat.bonus_bee_id;
        while count != 0 {
            if self.objects[id as usize / 2].state == State::Standby && id != bonus_bee {
                self.combat.sound_requests[0x13] = bonus_bee;
                self.c_1083(id, path);
                return;
            }
            id = id.wrapping_add(2);
            count -= 1;
        }
    }

    /// main:1C07. The first/second escort passes return directly to the task
    /// executive on success (the original removes one return address).
    fn case_bmbr_boss(&mut self) {
        if self.combat.capture_boss_flag == 0 {
            self.boss_escort_toggle = self.boss_escort_toggle.wrapping_add(1);
            if self.boss_escort_toggle & 1 == 0 {
                for id in (0x30..0x38).step_by(2) {
                    if self.objects[id / 2].state != State::Standby { continue; }
                    self.combat.capture_boss_flag = 1;
                    self.combat.capture_boss_id = id as u8;
                    self.j_1cae(id as u8, 2, 0, 0, 0x0454);
                    return;
                }
                return;
            }
        }
        let mut escorts = 0u8;
        for id in BOSS_ESCORT_IDS {
            let carry = id != self.combat.bonus_bee_id
                && self.objects[id as usize / 2].state == State::Standby;
            escorts = (escorts << 1) | u8::from(carry);
        }
        let saved = escorts;
        for b in (1..=4).rev() {
            let a = escorts & 7;
            let mut carry = a < 4;
            if a != 4 {
                carry = a < 3;
                if !carry && self.c_1c8d(b, escorts, 0, &mut carry) { return; }
            }
            escorts = (escorts >> 1) | (u8::from(carry) << 7);
        }
        escorts = saved;
        for b in (1..=4).rev() {
            let a = escorts & 7;
            let mut carry = false; // AND 7 clears carry before CALL NZ.
            if a != 0 && self.c_1c8d(b, escorts, 1, &mut carry) { return; }
            escorts = (escorts >> 1) | (u8::from(carry) << 7);
        }
        for id in (0x30..0x38).step_by(2) {
            if self.objects[id / 2].state == State::Standby {
                self.j_1ca0(id as u8, 2, 0, 0);
                return;
            }
        }
        for id in (0..8).step_by(2) {
            if self.objects[id / 2].state == State::Standby {
                self.c_1083(id as u8, 0x0444);
                return;
            }
        }
    }

    /// main:1C93. A failed CP 1 retains its carry for the caller's RR C.
    fn c_1c8d(&mut self, b: u8, c: u8, mode: u8, carry: &mut bool) -> bool {
        let mut a = b;
        if a & 2 != 0 { a ^= 1; }
        let id = ((a & 3) << 1) + 0x30;
        let state = self.objects[id as usize / 2].state;
        *carry = (state as u8) < 1;
        if state != State::Standby { return false; }
        self.j_1ca0(id, mode, b, c);
        true
    }

    /// main:1CA6, original training/game vector selection.
    fn j_1ca0(&mut self, id: u8, mode: u8, b: u8, c: u8) {
        self.j_1cae(id, mode, b, c, if self.enemy_enabled == 0 { 0x00f1 } else { 0x0411 });
    }

    /// main:1CB4. Queue the boss, record its two score attributes, then call
    /// the original escort helper once or twice, followed by a captured ship.
    fn j_1cae(&mut self, id: u8, mode: u8, mut b: u8, mut c: u8, path: u16) {
        let rotation = (id & 2) << 6;
        self.boss_sorties[0] = Sortie { object_id: id | rotation, path };
        b = b.wrapping_add(1);
        let attribute = (id & 7) as usize;
        self.combat.boss_score_attributes[attribute..attribute + 2]
            .copy_from_slice(&BOSS_SORTIE_ATTRIBUTES[mode as usize * 2..mode as usize * 2 + 2]);
        let mut queue = 1usize;
        if mode != 2 {
            if mode != 1 { self.c_1d03(&mut b, &mut c, &mut queue, rotation, path); }
            self.c_1d03(&mut b, &mut c, &mut queue, rotation, path);
        }
        let captured = self.boss_sorties[0].object_id & 7;
        // The ASM reads the STATE; the older C mistakenly reads motion_slot.
        if self.objects[captured as usize / 2].state != State::Standby { return; }
        let mut queue = 1usize;
        while self.boss_sorties[queue].object_id != 0xff { queue += 1; }
        self.l_1d16(queue, captured, rotation, path);
    }

    /// main:1D09, rotate flags right (not a 16-bit logical shift).
    fn c_1d03(&mut self, b: &mut u8, c: &mut u8, queue: &mut usize, rotation: u8, path: u16) {
        let first = *c & 1 != 0;
        *c = c.rotate_right(1);
        if !first {
            *b = b.wrapping_sub(1);
            let second = *c & 1 != 0;
            *c = c.rotate_right(1);
            if !second { *b = b.wrapping_sub(1); }
        }
        let index = *b;
        *b = b.wrapping_sub(1);
        self.l_1d16(*queue, BOSS_ESCORT_IDS[index as usize], rotation, path);
        *queue += 1;
    }

    /// main:1D1C, shared by escort calls and the rogue-fighter tail.
    fn l_1d16(&mut self, queue: usize, id: u8, rotation: u8, path: u16) {
        self.boss_sorties[queue] = Sortie { object_id: (id & 0x7f) | rotation, path };
    }

    /// main:1079, explicit rotation flag carried by the queued ID.
    pub fn c_1079(&mut self, id: u8, path: u16) {
        self.j_108a(id & 0x7f, path, (id & 0x80).wrapping_add(1));
    }
    /// main:1083, bit 1 of the drone's object ID selects rotation direction.
    pub fn c_1083(&mut self, id: u8, path: u16) {
        self.j_108a(id, path, ((id.rotate_right(2)) & 0x80).wrapping_add(1));
    }
    /// main:108A..110B. Only original written fields are changed; an available
    /// motion slot's other bytes are retained until their original stores.
    fn j_108a(&mut self, id: u8, path: u16, flags: u8) {
        let Some(slot) = self.motions.iter().position(|m| !m.active()) else { return; };
        let object = &mut self.objects[id as usize / 2];
        let motion = &mut self.motions[slot];
        motion.path = path;
        motion.duration = 1;
        motion.angle = 0x0100;
        motion.object_id = id;
        object.state = State::Homing;
        object.motion_slot = slot as u8;
        let y = if self.flip_screen & 1 == 0 { 0x161u16.wrapping_sub(object.y) } else { object.y };
        motion.y = y.wrapping_shl(7);
        let x = if self.flip_screen & 1 == 0 { object.x } else { !object.x.wrapping_add(13) };
        motion.x = (x as u16) << 7;
        motion.flags = flags;
        motion.bomb_counter = 0x1e;
        motion.bomb_flags = if self.enemy_enabled == 0 { 0 } else { self.bomber_control[8] };
    }
}
