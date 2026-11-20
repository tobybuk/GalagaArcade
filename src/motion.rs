//! Galaga's own flight-vector language and fixed-point motion routines.
//! This is game logic, not a Z80 instruction interpreter. Tokens are read only
//! from individually identified flight DATA tables extracted from the ROM.
use crate::{rom_data::*, Game, Object, State, FIGHTER};

#[derive(Clone, Copy, Default, Debug)]
pub struct Motion {
    pub y: u16, pub x: u16, pub angle: u16,
    pub home_y: u8, pub home_x: u8,
    pub path: u16,
    pub odd_displacement: u8, pub even_displacement: u8,
    pub rotation: i8, pub duration: u8,
    pub bomb_counter: u8, pub bomb_flags: u8,
    pub object_id: u8, pub offset_x: u8, pub offset_y: u8,
    pub flags: u8,
}

pub fn flight_byte(address: u16) -> Result<u8, u16> {
    match address {
        0x001d..=0x0033 => Ok(ENTRANCE_TOP[(address - 0x001d) as usize]),
        0x0067..=0x0083 => Ok(ENTRANCE_SIDE[(address - 0x0067) as usize]),
        0x004b..=0x0065 => Ok(FLIGHT_TRANSIENT_TOP[(address - 0x004b) as usize]),
        0x0084..=0x00f7 => Ok(FLIGHT_EARLY[(address - 0x0084) as usize]),
        0x0160..=0x0512 => Ok(FLIGHT_PATTERNS[(address - 0x0160) as usize]),
        0x0fda..=0x0ffc => Ok(FLIGHT_LATE[(address - 0x0fda) as usize]),
        _ => Err(address),
    }
}

/// sub:0x0EAE (c_0EAA). The original division defines zero-divisor as 0xffff.
pub fn divide(numerator: u16, denominator: u8) -> u16 {
    divide_with_remainder(numerator, denominator).0
}

pub fn divide_with_remainder(numerator: u16, denominator: u8) -> (u16, u8) {
    // Retain the original 17-pass restoring-division flow, including its
    // zero-divisor behaviour. A native '/' was equivalent for all inputs,
    // but is unnecessary divergence from the requested original program flow.
    let mut quotient = numerator;
    let mut remainder = 0u8;
    let mut carry = 0u16;
    for _ in 0..17 {
        let doubled = remainder as u16 + remainder as u16 + carry;
        remainder = doubled as u8;
        let quotient_bit = if doubled & 0x100 != 0 || remainder >= denominator {
            remainder = remainder.wrapping_sub(denominator);
            1u32
        } else { 0u32 };
        let shifted = quotient as u32 + quotient as u32 + quotient_bit;
        quotient = shifted as u16;
        carry = ((shifted >> 16) & 1) as u16;
    }
    (quotient, remainder)
}

/// sub:0E9B (c_0E97). Preserve the original shift/add multiply subroutine.
pub fn multiply(mut factor: u8, value: u8) -> u16 {
    let mut addend = value as u16;
    let mut result = 0u16;
    loop {
        if factor & 1 != 0 { result = result.wrapping_add(addend); }
        addend = addend.wrapping_shl(1);
        factor >>= 1;
        if factor == 0 { return result; }
    }
}

/// sub:0x0E5F (c_0E5B). Original octant selection and integer division.
/// The returned angle is doubled; the caller shifts it right once.
pub fn home_angle(x: u8, y: u8, target_x: u8, target_y: u8) -> u16 {
    let mut horizontal = target_x.wrapping_sub(x);
    let mut octant = 0u8;
    if x > target_x { octant = 1; horizontal = horizontal.wrapping_neg(); }
    let mut vertical = target_y.wrapping_sub(y);
    if y > target_y {
        octant ^= 1;
        octant |= 2;
        vertical = vertical.wrapping_neg();
    }
    let swap = horizontal > vertical;
    octant = (octant << 1) | (((swap as u8) ^ octant ^ 1) & 1);
    let (minor, major) = if swap { (vertical, horizontal) } else { (horizontal, vertical) };
    let quotient = divide((minor as u16) << 8, major);
    let low = if (((quotient >> 8) as u8) ^ octant) & 1 != 0 {
        !(quotient as u8)
    } else { quotient as u8 };
    u16::from_le_bytes([low, octant])
}

/// sub:0x0CA7..0x0D02. Preserve the original piecewise integer trajectory.
/// A trigonometric replacement produces different paths and is not used.
pub fn coordinate_step(x: u16, y: u16, angle: u16, displacement: u8) -> (u16, u16) {
    let angle_hi = (angle >> 8) as u8;
    let horizontal = ((angle & 0x80 != 0) ^ (angle_hi & 1 != 0)) == false;
    let doubled_hi = (((angle & 0x3ff) << 1) >> 8) as u8;
    let signed_major = if doubled_hi.wrapping_add(1) & 4 != 0 {
        (displacement as i16).wrapping_neg()
    } else { displacement as i16 };
    let major = signed_major * 128;
    let fraction = if angle & 0x80 == 0 { angle as u8 & 0x7f } else { !angle as u8 & 0x7f };
    let minor = multiply(displacement, fraction);
    let minor_negative = (doubled_hi ^ 2).wrapping_sub(1) & 4 != 0;
    let minor = if minor_negative { minor.wrapping_neg() } else { minor };
    if horizontal { (x.wrapping_add(major as u16), y.wrapping_add(minor)) }
    else { (x.wrapping_add(minor), y.wrapping_add(major as u16)) }
}

/// sub:0x0C46..0x0CA6. Six drawn orientations per quadrant and hardware flips.
pub fn sprite_orientation(angle: u16) -> (u8, u8) {
    let hi = (angle >> 8) as u8;
    let low = if angle & 0x100 != 0 { !(angle as u8) } else { angle as u8 };
    let rounded = low as u16 + 21;
    let tile = if rounded > 255 { 6 } else {
        let a = rounded as u8 >> 1;
        ((a as u16 + (a >> 1) as u16) >> 5) as u8 & 7
    };
    let flip_x = (hi >> 1) & 1;
    let flags = (((hi ^ flip_x).wrapping_add(1) << 1) | flip_x) & 3;
    (tile, flags)
}

impl Motion {
    pub fn active(&self) -> bool { self.flags & 1 != 0 }
}

fn flight_word(address: u16) -> Result<u16, u16> {
    Ok(u16::from_le_bytes([flight_byte(address)?, flight_byte(address.wrapping_add(1))?]))
}

impl Game {
    /// sub:090E. Each case retains the original distinction between continuing
    /// token processing, continuing motion, and returning to the pool traversal.
    /// A local motion/object pair permits Rust borrowing without changing the
    /// original shared slot visibility or order of child-slot allocation.
    pub fn mctl_fltpn_dspchr(&mut self, motion: &mut Motion, object: &mut Object, slot: usize) -> Result<(), u16> {
        motion.duration = motion.duration.wrapping_sub(1);
        if motion.duration == 0 {
            let mut step_ready = false;
            for _ in 0..32 {
                let token = flight_byte(motion.path)?;
                match token {
                    0xff => {
                        self.case_0e49(motion, object);
                        return Ok(());
                    }
                    0xfe => {
                        // case_0B16: formation departure, 8-way duration lookup.
                        let mut a = self.sprite_ram[FIGHTER].x;
                        if a == 0 { a = 0x80; }
                        let direction = ((self.flip_screen.rotate_right(1)) ^ motion.flags) & 0x80 != 0;
                        if !direction { a = a.wrapping_neg().wrapping_add(0xf2); }
                        a = a.wrapping_add(0x0e);
                        let index = (divide((a as u16) << 8, 0x1e) >> 8) as u16;
                        motion.duration = flight_byte(motion.path + index + 1)?;
                        motion.path += 9;
                        step_ready = true;
                        break;
                    }
                    0xfd => { motion.path = flight_word(motion.path + 1)?; }
                    0xfc => {
                        motion.home_y = flight_byte(motion.path + 1)?;
                        motion.home_x = 0;
                        motion.flags |= 0x20;
                        motion.path += 2;
                        step_ready = true;
                        break;
                    }
                    0xfb => { self.case_0aa0(motion, object); motion.path += 1; }
                    0xfa => {
                        if self.main_task_enable[0x1d].wrapping_sub(1) & self.continuous_bombing == 0 {
                            motion.path = flight_word(motion.path + 1)?;
                        } else { motion.path += 3; }
                    }
                    0xf9 => {
                        let column = FORMATION_ORDINALS[motion.object_id as usize + 1] as usize / 2;
                        let mut a = self.formation.coordinates[column] as u8;
                        if self.flip_screen & 1 != 0 { a = a.wrapping_add(14).wrapping_neg(); }
                        motion.x = (motion.x & 0xff) | (((a >> 1) as u16) << 8);
                        if self.continuous_bombing != 0 { self.combat.sound_requests[0x13] = self.continuous_bombing; }
                        motion.path += 1;
                        motion.duration = motion.duration.wrapping_add(1);
                        return Ok(());
                    }
                    0xf8 => {
                        motion.y = (motion.y & 0xff) | 0x9c00;
                        motion.path += 1;
                        motion.duration = motion.duration.wrapping_add(1);
                        return Ok(());
                    }
                    0xf7 => {
                        if motion.object_id & 0x38 == 0x38 { motion.path = flight_word(motion.path + 1)?; }
                        else { motion.path += 3; }
                    }
                    0xf6 => {
                        let mut a = flight_byte(motion.path + 1)?;
                        if motion.flags & 0x80 != 0 { a = a.wrapping_add(0x80).wrapping_neg(); }
                        motion.angle = (a as u16) << 2;
                        motion.bomb_counter = 0x1e;
                        motion.bomb_flags = self.bomber_control[8];
                        motion.path += 2;
                        motion.duration = motion.duration.wrapping_add(1);
                        return Ok(());
                    }
                    0xf5 => { object.state = State::Pattern; motion.path += 1; }
                    0xf4 => {
                        // case_0A53, target capture position and enable the
                        // original capture-approach task (main:21CB).
                        let mut a = (self.objects[FIGHTER].x.wrapping_add(3) & 0xf8).wrapping_add(1);
                        if a < 0x29 { a = 0x29; }
                        if a >= 0xca { a = 0xc9; }
                        if self.flip_screen & 1 != 0 { a = !a.wrapping_add(13); }
                        self.combat.capture_status[0] = a;
                        motion.angle = home_angle((motion.x >> 8) as u8, (motion.y >> 8) as u8, a >> 1, 0x48) >> 1;
                        self.combat.capture_status[1] = 0;
                        self.main_task_enable[0x19] = 1;
                        self.capture_motion_slot = slot as u8;
                        motion.path += 1;
                    }
                    0xf3 => {
                        // case_0A01: signed quarter-distance targeting.
                        let mut a = self.objects[FIGHTER].x;
                        if a < 0x1e { a = 0x1e; }
                        if a >= 0xd1 { a = 0xd1; }
                        if self.flip_screen & 1 != 0 { a = a.wrapping_add(14).wrapping_neg(); }
                        let (difference, carry) = (a >> 1).overflowing_sub((motion.x >> 8) as u8);
                        a = (difference >> 1) | (u8::from(carry) << 7);
                        if motion.flags & 0x80 != 0 { a = a.wrapping_neg(); }
                        a = a.wrapping_add(0x18);
                        if a & 0x80 != 0 { a = 0; }
                        if a >= 0x30 { a = 0x2f; }
                        let index = (divide((a as u16) << 8, 6) >> 8) as u16;
                        motion.duration = flight_byte(motion.path + index + 1)?;
                        motion.path += 9;
                        step_ready = true;
                        break;
                    }
                    0xf2 => { self.case_097b(motion, object)?; motion.path += 3; }
                    0xf1 => {
                        let row = FORMATION_ORDINALS[motion.object_id as usize] as usize / 2;
                        motion.y = (motion.y & 0xff) | ((FORMATION_ORIGIN[row].wrapping_add(0x20) as u16) << 8);
                        motion.path += 1;
                        motion.duration = motion.duration.wrapping_add(1);
                        return Ok(());
                    }
                    0xf0 | 0xef => {
                        let parameter = if token == 0xf0 { 8 } else { 9 };
                        if self.stage_parameters[parameter] != 0 { motion.path = flight_word(motion.path + 1)?; }
                        else { motion.path += 3; }
                        motion.duration = motion.duration.wrapping_add(1);
                        return Ok(());
                    }
                    0x00..=0xee => {
                        motion.odd_displacement = token & 15;
                        motion.even_displacement = token >> 4;
                        let rotation = flight_byte(motion.path + 1)?;
                        motion.rotation = if motion.flags & 0x80 != 0 { rotation.wrapping_neg() as i8 } else { rotation as i8 };
                        motion.duration = flight_byte(motion.path + 2)?;
                        motion.path += 3;
                        step_ready = true;
                        break;
                    }
                }
            }
            if !step_ready { return Err(motion.path); }
        }
        self.mctl_fltpn_cont(motion, object);
        Ok(())
    }

    /// sub:0AA0: convert to formation-relative coordinates, preserving the
    /// original helper call and signed offset shifts before home-angle setup.
    fn case_0aa0(&mut self, motion: &mut Motion, object: &mut Object) {
        object.state = State::Homing;
        let id = motion.object_id as usize;
        let row = FORMATION_ORDINALS[id] as usize / 2;
        let column = FORMATION_ORDINALS[id + 1] as usize / 2;
        motion.offset_x = self.formation.offsets[column];
        motion.offset_y = self.formation.offsets[row];
        motion.home_x = FORMATION_ORIGIN[column] >> 1;
        motion.home_y = FORMATION_ORIGIN[row];
        let mut x = motion.offset_x;
        let mut y = motion.offset_y;
        if self.flip_screen != 0 { x = x.wrapping_neg(); y = y.wrapping_neg(); }
        motion.y = motion.y.wrapping_add(((y as i8 as i16) * 128) as u16);
        motion.x = motion.x.wrapping_sub(((x as i8 as i16) * 128) as u16);
        motion.angle = home_angle((motion.x >> 8) as u8, (motion.y >> 8) as u8, motion.home_x, motion.home_y) >> 1;
        motion.flags |= 0x40;
    }

    /// sub:097B: copy the exact original motion fields to a free transient,
    /// selecting object slots upwards and motion slots downwards.
    fn case_097b(&mut self, parent: &Motion, object: &Object) -> Result<(), u16> {
        let Some(id) = (28..32).find(|&n| self.objects[n].state as u8 & 0x80 != 0) else { return Ok(()); };
        self.objects[id].code = object.code;
        self.objects[id].colour = object.colour;
        let Some(slot) = (0..12).rev().find(|&n| !self.motions[n].active()) else { return Ok(()); };
        let child = &mut self.motions[slot];
        child.y = parent.y; child.x = parent.x; child.angle = parent.angle;
        child.rotation = parent.rotation;
        child.bomb_counter = parent.bomb_counter; child.bomb_flags = parent.bomb_flags;
        child.flags = parent.flags;
        child.path = flight_word(parent.path + 1)?;
        child.odd_displacement = 1; child.even_displacement = 2; child.duration = 1;
        child.object_id = (id * 2) as u8;
        self.objects[id].state = State::Homing;
        self.objects[id].motion_slot = slot as u8;
        Ok(())
    }

    /// sub:0C05 -> 0C46,0CA7 or 0E0C -> 0D03.
    fn mctl_fltpn_cont(&mut self, motion: &mut Motion, object: &mut Object) {
        let near = |a: u8, b: u8| matches!(a.wrapping_sub(b), 0 | 1 | 255);
        if motion.flags & 0x40 != 0
            && near((motion.y >> 8) as u8, motion.home_y)
            && near((motion.x >> 8) as u8, motion.home_x) {
            self.l_0e08_imhome(motion, object);
        } else {
            if motion.flags & 0x20 != 0 {
                let difference = ((motion.y >> 8) as u8).wrapping_sub(motion.home_y);
                if difference == 0 || difference == 255 { motion.duration = 1; motion.flags &= !0x20; }
            }
            let (tile, flags) = sprite_orientation(motion.angle);
            object.code = (object.code & 0xf8) | tile;
            object.flags = flags;
            let displacement = if self.counters[0] & 1 != 0 { motion.odd_displacement } else { motion.even_displacement };
            (motion.x, motion.y) = coordinate_step(motion.x, motion.y, motion.angle, displacement);
            motion.angle = motion.angle.wrapping_add(motion.rotation as i16 as u16);
        }
        self.mctl_posn_set(motion, object);
    }

    /// sub:0E0C (older l_0E08), including the bonus-bee class restoration.
    fn l_0e08_imhome(&mut self, motion: &mut Motion, object: &mut Object) {
        motion.flags &= !1;
        object.state = State::HomeReturn;
        motion.y &= 0xff00; motion.x &= 0xff00;
        if object.colour.wrapping_add(1) & 7 >= 5 {
            let packed = self.bonus_bee_class;
            object.code = (packed & 0xf8).wrapping_add(6);
            object.colour = packed & 7;
            self.combat.bonus_bee_id = 1;
        }
        motion.y = (motion.home_y as u16) << 8;
        motion.x = (motion.home_x as u16) << 8;
    }

    /// sub:0E4D. The original clears the active bit while retaining other flags.
    pub fn case_0e49(&mut self, motion: &mut Motion, object: &mut Object) {
        motion.flags &= !1;
        object.x = 0;
        object.state = State::Inactive;
    }

    /// sub:0D03..0D5C, sprite registers then the original bomb-drop checks.
    fn mctl_posn_set(&mut self, motion: &mut Motion, object: &mut Object) {
        object.x = (motion.x >> 7) as u8;
        if self.flip_screen & 1 != 0 { object.x = !object.x.wrapping_add(13); }
        object.y = if self.flip_screen & 1 == 0 { 0x161u16.wrapping_sub(motion.y >> 7) } else { motion.y >> 7 };
        if motion.flags & 0x40 != 0 {
            object.x = object.x.wrapping_add(motion.offset_x);
            object.y = object.y.wrapping_add(motion.offset_y as i8 as i16 as u16);
        }
        object.y &= 0x1ff;
        self.mctl_bomb_drop(motion, object);
    }

    /// sub:0D5D..0DFE. Preserve the low numerator byte left by the bomb
    /// register pointer; dropping it changes some horizontal rates.
    fn mctl_bomb_drop(&mut self, motion: &mut Motion, object: &Object) {
        motion.bomb_counter = motion.bomb_counter.wrapping_sub(1);
        if motion.bomb_counter != 0 { return; }
        let enabled = motion.bomb_flags & 1 != 0;
        motion.bomb_flags >>= 1;
        if enabled && motion.y >> 8 >= 0x4c && self.main_task_enable[0x15] != 0 && self.game_timers[1] == 0 {
            if let Some(bomb) = (52..60).find(|&n| self.objects[n].state == State::Inactive) {
                self.objects[bomb].state = State::Bomb;
                // rev. B adds this CTRL-byte store before copying position.
                self.objects[bomb].flags = 1;
                self.objects[bomb].x = object.x;
                self.objects[bomb].y = object.y;
                let (mut dx, left) = self.objects[FIGHTER].x.overflowing_sub(object.x);
                if left { dx = dx.wrapping_neg(); }
                let target_y = if self.flip_screen == 0 { 0x95u8 } else { 0x1c };
                let (mut dy, above) = target_y.overflowing_sub((object.y >> 1) as u8);
                if above { dy = dy.wrapping_neg(); }
                let quotient = divide(((dx as u16) << 8) | (bomb * 2 + 1) as u16, dy);
                let scaled = ((quotient >> 2).wrapping_add(quotient)) >> 2;
                let rate = if scaled >= 0x60 { 0x60 } else { scaled as u8 };
                self.bomb_rates[bomb - 52] = [(rate >> 1) | (u8::from(left) << 7), 0];
            }
        }
        motion.bomb_counter = self.bomb_reload;
    }
}
