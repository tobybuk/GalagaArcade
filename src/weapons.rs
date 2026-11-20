//! Original fighter-control and rocket routines, for the normal single fighter.
use crate::{Game, State, FIGHTER, ROCKET_0};

impl Game {
    /// main:0x1F9C (c_1F92). The actual code alternates steps 1,2,1,2;
    /// the annotation's prose incorrectly suggests 1 followed by constant 2.
    pub fn fighter_control(&mut self, active_low_input: u8) {
        let mut controls = active_low_input & 0x0a;
        if controls == 0x0a {
            self.fighter_phase = 0;
            return;
        }
        if self.flip_screen & 1 != 0 { controls ^= 0x0a; }
        self.fighter_phase ^= 1;
        let delta = if self.fighter_phase == 0 { 2 } else { 1 };
        let fighter = &mut self.objects[FIGHTER];
        if fighter.x == 0 { return; }
        if controls & 2 == 0 {
            if fighter.x >= 0xd1 && self.two_fighters & 1 != 0 { return; }
            if fighter.x >= 0xe1 { return; }
            fighter.x = fighter.x.wrapping_add(delta);
        } else {
            if fighter.x < 0x12 { return; }
            fighter.x = fighter.x.wrapping_sub(delta);
        }
        if self.two_fighters & 1 != 0 {
            self.objects[FIGHTER - 1].x = self.objects[FIGHTER].x.wrapping_add(15);
        }
    }

    /// main:0x1F19 (c_1F0F). Two simultaneously available rocket slots.
    pub fn launch_rocket(&mut self) {
        let Some(slot) = (ROCKET_0..ROCKET_0 + 2).find(|&i| self.objects[i].x == 0) else { return; };
        let fighter = self.objects[FIGHTER];
        if fighter.odd_control()&4!=0 {return;} // main:1F2A, original fire-inhibit bit
        let mut code = fighter.code & 7;
        let rocket = &mut self.objects[slot];
        rocket.x = fighter.x;
        rocket.y = fighter.y;
        rocket.write_odd_control(fighter.odd_control());
        rocket.flags = fighter.flags | ((self.two_fighters & 1) << 3);
        rocket.code = if code >= 5 { 0x30 } else if code >= 2 { 0x31 } else { 0x33 };
        if code >= 4 { code = 6u8.wrapping_sub(code).wrapping_add(0x40); }
        let mut flips = (fighter.flags << 5) & 0x60;
        if self.flip_screen == 0 { flips ^= 0x60; }
        self.rocket_attributes[slot - ROCKET_0] = flips | (code << 1);
        rocket.state = State::Bomb;
        self.combat.sound_requests[0x0f] = 1;
        self.shots = self.shots.wrapping_add(1);
    }

    /// sub:0x06F5..0x0769. Update both rocket slots with the original 6-pixel
    /// major displacement, axis selection, signs and off-screen limits.
    pub fn rocket_motion(&mut self) {
        for n in 0..2 {
            let rocket = &mut self.objects[ROCKET_0 + n];
            if rocket.x == 0 { continue; }
            let attr = self.rocket_attributes[n];
            let (mut dx, mut dy) = if attr & 0x80 != 0 { (attr & 7, 6u8) } else { (6u8, attr & 7) };
            if attr & 0x40 != 0 { dx = dx.wrapping_neg(); }
            if attr & 0x20 != 0 { dy = dy.wrapping_neg(); }
            rocket.x = rocket.x.wrapping_add(dx);
            let mut in_bounds = rocket.x < 240;
            if in_bounds {
                let y = rocket.y.wrapping_add(dy as i8 as i16 as u16);
                rocket.y = y & 0x1ff;
                // Original CP 0x9C / JR NC tests the scaled nine-bit register:
                // Y==312 is already outside. The older C aid used <=312.
                in_bounds = (20..156).contains(&(rocket.y >> 1));
            }
            if !in_bounds {
                rocket.x = 0;
                rocket.flags = 0;
            } else {
                self.hitd_det_rckt(ROCKET_0 + n);
            }
        }
    }
}
