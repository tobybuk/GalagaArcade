//! main:0728,07D8: score supervisor and decimal tile increment.
//! Scores live in the original tile RAM. There is no replacement numeric score
//! store. This follows the assembly; the older C score supervisor is incomplete.
use crate::{rom_data::*, Game};

impl Game {
    /// Initial tile writes from c_textout_1uphighscore_onetime, playfield clear
    /// and gctl_init_puts. The surrounding boot executive remains unported.
    pub fn initialise_score_tiles(&mut self) {
        self.tile_ram.fill(0x24);
        self.colour_ram.fill(0);
        self.colour_ram[..0x40].fill(3);
        self.colour_ram[0x3bf..0x3df].fill(4);
        self.colour_ram[0x3df..0x3ff].fill(0x4e);
        self.tile_ram[0x3ed..0x3f2].copy_from_slice(&HIGH_SCORE_INITIAL[..5]);
        self.tile_ram[0x3cb..0x3dc].copy_from_slice(&HEADER_TILES);
        self.gctl_init_puts(0, 0x3f8);
        self.gctl_init_puts(if self.two_player_game { 0 } else { 2 }, 0x3e3);
        self.draw_resv_ships();
    }

    /// main:047D (gctl_init_puts). Preserve the two separate copy operations.
    pub(crate) fn gctl_init_puts(&mut self, source_offset: usize, destination: usize) {
        self.tile_ram[destination..destination + 7]
            .copy_from_slice(&SCORE_INITIAL[source_offset..source_offset + 7]);
        self.tile_ram[0x3c3..0x3c7].copy_from_slice(&SCORE_INITIAL[2..6]);
    }

    /// main:07D8 c_scoreman_incr_add. INC L wraps within the original page.
    pub fn c_scoreman_incr_add(&mut self, mut low_address: u8, increment: u8) {
        if increment == 0 { return; }
        let mut a = increment.wrapping_add(self.tile_ram[0x300 + low_address as usize]);
        if a >= 0x24 { a = a.wrapping_sub(0x24); }
        if a < 10 {
            self.tile_ram[0x300 + low_address as usize] = a;
            return;
        }
        a = a.wrapping_sub(10);
        loop {
            self.tile_ram[0x300 + low_address as usize] = a;
            low_address = low_address.wrapping_add(1);
            a = self.tile_ram[0x300 + low_address as usize];
            if a == 0x24 { a = 0; }
            if a != 9 {
                self.tile_ram[0x300 + low_address as usize] = a.wrapping_add(1);
                return;
            }
            a = 0;
        }
    }

    /// main:0728 gctl_supv_score, called from the original background runner
    /// before its wait for the next video period, not from the interrupt table.
    pub fn gctl_supv_score(&mut self) {
        let tens = if self.player_number == 0 { 0xf9u8 } else { 0xe4u8 };
        for (index, factor) in SCORE_FACTORS.iter().rev().copied().enumerate() {
            while self.combat.hit_multipliers[index] != 0 {
                self.combat.hit_multipliers[index] = self.combat.hit_multipliers[index].wrapping_sub(1);
                self.c_scoreman_incr_add(tens, factor & 15);
                self.c_scoreman_incr_add(tens.wrapping_add(1), factor >> 4);
            }
        }
        // main:0762..078D. Keep the original byte comparison and copy direction,
        // including its treatment of the blank character (0x24).
        let mut source = tens.wrapping_add(4);
        let mut destination = 0xf2u8;
        let mut remaining = 6u8;
        while remaining != 0 {
            let mut a = self.tile_ram[0x300 + source as usize]
                .wrapping_sub(self.tile_ram[0x300 + destination as usize])
                .wrapping_add(9);
            let mut copy = a >= 0xe5;
            if !copy {
                a = a.wrapping_sub(10);
                copy = a < 9;
                if !copy && a.wrapping_add(1) != 0 { break; }
            }
            if copy {
                while remaining != 0 {
                    self.tile_ram[0x300 + destination as usize] = self.tile_ram[0x300 + source as usize];
                    destination = destination.wrapping_sub(1);
                    source = source.wrapping_sub(1);
                    remaining -= 1;
                }
                break;
            }
            destination = destination.wrapping_sub(1);
            source = source.wrapping_sub(1);
            remaining -= 1;
        }
        // main:078E..07D7. Original bonus threshold, reserve count, sound request
        // and service BCD counter. The 0x80 bit in the second bonus is retained.
        let digit = |tile: u8| if tile == 0x24 { 0 } else { tile };
        let a = digit(self.tile_ram[0x300 + tens.wrapping_add(4) as usize]) & 0x3f;
        let c = a.rotate_left(1).wrapping_add(a.rotate_left(3));
        let ten_thousands = digit(self.tile_ram[0x300 + tens.wrapping_add(3) as usize]);
        if ten_thousands.wrapping_add(c) != self.next_bonus { return; }
        let second = self.machine_bonus[1];
        self.next_bonus = if self.next_bonus < second & 0x7f {
            second & 0x7f
        } else { self.next_bonus.wrapping_add(second) };
        self.combat.sound_requests[0x0a] = self.next_bonus;
        self.reserve_fighters = self.reserve_fighters.wrapping_add(1);
        self.draw_resv_ships();
        let (low, carry) = increment_bcd(self.bonus_count_bcd[1]);
        self.bonus_count_bcd[1] = low;
        if carry { self.bonus_count_bcd[0] = increment_bcd(self.bonus_count_bcd[0]).0; }
    }

    /// main:137E (draw_resv_ships in this ROM revision), four calls in the
    /// original order. Each helper preserves its incoming tile pointer.
    pub fn draw_resv_ships(&mut self) {
        // Supplied rev. B differs from the older annotated routine: eight
        // slots, and existing stage-token tiles 0x36..0x49 are protected.
        let empty_from = (!self.reserve_fighters).wrapping_add(9);
        let mut tile = 0x49u8;
        for address in [0x1du8, 0x1c, 0x3d, 0x3c] {
            tile = tile.wrapping_add(1);
            let mut character = tile;
            let mut low = address;
            for a in (1..=8).rev() {
                if a == empty_from { character = 0x24; }
                let existing = self.tile_ram[low as usize];
                if !(0x36..0x4a).contains(&existing) { self.tile_ram[low as usize] = character; }
                low = low.wrapping_sub(2);
            }
        }
    }

    /// main:0935 -> gctl_1up2up_displ -> c_095F.
    pub fn f_0935(&mut self) {
        self.c_093c(self.counters[0] >> 4);
    }

    pub fn c_093c(&mut self,counter:u8) {
        if self.game_state != 3 { return; }
        for player in 0..=usize::from(self.two_player_game) {
            let mask = if player == 0 { !self.player_number } else { self.player_number };
            let destination = if player == 0 { 0x3d9 } else { 0x3c4 };
            let text = if mask & counter & 1 != 0 { [0x24; 3] } else { [0x19, 0x1e, player as u8 + 1] };
            self.tile_ram[destination..destination + 3].copy_from_slice(&text);
        }
    }

    /// Read-only presentation/inspection helper; the game logic uses tile RAM.
    pub fn score_value(&self, player: u8) -> u32 {
        let ones = if player == 0 { 0x3f8 } else { 0x3e3 };
        (0..6).rev().fold(0, |value, i| {
            let tile = self.tile_ram[ones + i];
            value * 10 + if tile < 10 { tile as u32 } else { 0 }
        })
    }
}

pub fn increment_bcd(value: u8) -> (u8, bool) {
    let sum = value as u16 + 1;
    let low_corrected = sum + if sum & 15 > 9 { 6 } else { 0 };
    let corrected = low_corrected + if low_corrected > 0x99 { 0x60 } else { 0 };
    (corrected as u8, corrected > 255)
}
