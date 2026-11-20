//! Original main text-output routines and identified ROM strings.
use crate::{motion::divide_with_remainder, rom_data::TEXT_DATA, Game};

fn text_byte(address: u16) -> u8 { TEXT_DATA[(address - 0x13f1) as usize] }
fn text_word(address: u16) -> u16 {
    u16::from_le_bytes([text_byte(address), text_byte(address.wrapping_add(1))])
}

impl Game {
    /// main:104E, original shift/add 16-bit product helper (BC/DE saves are
    /// represented by ordinary Rust arguments rather than a simulated CPU).
    pub fn c_104e_mul_16_8(mut value: u16, mut factor: u8) -> u16 {
        let mut product = 0u16;
        loop {
            let carry = factor & 1 != 0;
            factor >>= 1;
            if carry { product = product.wrapping_add(value); }
            value = value.wrapping_shl(1);
            if factor == 0 { return product; }
        }
    }

    /// main:0B06, low-byte product and the original high-byte return in A.
    fn c_0b06(value: u16) -> (u16, u8) {
        let product = Self::c_104e_mul_16_8(value, 10);
        ((product as u8) as u16, (product >> 8) as u8)
    }

    /// main:0A72..0B05. Preserve the original scaled division, nibble rotates,
    /// shift/add products and BCD rounding. A floating-point ratio differs.
    #[allow(unused_assignments)] // RLD's A result is discarded by the original next helper call.
    pub fn c_0a72_puts_hitmiss_ratio(&mut self) -> u16 {
        let mut temporary = [self.combat.squad_remaining, self.combat.squad_bonus_increment];
        if self.shots == 0 { temporary = [0, 0]; }
        else {
            let mut hits = self.combat.hits;
            let mut shots = self.shots;
            while (shots | hits) & 0x8000 == 0 { shots <<= 1; hits <<= 1; }
            let divisor = (shots >> 8) as u8;
            let (mut first, remainder) = Self::c_divmod(hits, divisor);
            let (mut second, _) = Self::c_divmod((remainder as u16) << 8, divisor);
            let mut a = (first >> 8) as u8;
            first &= 0xff;
            let mut pointer = 0usize;
            for b in (1..=4).rev() {
                // RLD exchanges A's low nibble with the addressed RAM byte.
                let byte = temporary[pointer];
                temporary[pointer] = (byte << 4) | (a & 15);
                a = (a & 0xf0) | (byte >> 4);
                if b & 1 != 0 { pointer += 1; }
                let (low, high) = Self::c_0b06(first);
                first = low;
                let (low, carry_byte) = Self::c_0b06(second);
                second = low;
                first = first.wrapping_add(carry_byte as u16);
                a = high.wrapping_add((first >> 8) as u8);
                first &= 0xff;
            }
            if a >= 5 {
                let (low, carry) = bcd_add_one(temporary[1]);
                temporary[1] = low;
                if carry { temporary[0] = bcd_add_one(temporary[0]).0; }
            }
        }
        let mut destination = 0x8138u16;
        let mut pointer = 0usize;
        let mut significant = false;
        for b in (1..=4).rev() {
            if b == 1 {
                self.tile_ram[(destination & 0x3ff) as usize] = 0x2a;
                destination = destination.wrapping_sub(0x20);
            }
            let digit = temporary[pointer] >> 4;
            temporary[pointer] <<= 4;
            if b & 1 != 0 { pointer += 1; }
            if digit != 0 || significant {
                significant = true;
                self.tile_ram[(destination & 0x3ff) as usize] = digit;
                destination = destination.wrapping_sub(0x20);
            }
            if b == 3 { significant = true; }
        }
        self.combat.squad_remaining = temporary[0];
        self.combat.squad_bonus_increment = temporary[1];
        destination
    }

    /// main:13B3, RST 30 / j_string_out_pe at 13B5. String index is one-based.
    /// Return the original final tile address for callers that append digits.
    pub fn c_string_out(&mut self, index: u8, destination: Option<u16>) -> u16 {
        let mut source = text_word(0x13f1 + (index as u16 - 1) * 2);
        let mut destination = destination.unwrap_or_else(|| text_word(source - 2));
        let colour = text_byte(source);
        source += 1;
        loop {
            let character = text_byte(source);
            if character == 0x2f { return destination; }
            let (mut tile, carry) = character.overflowing_sub(0x30);
            if carry { tile = 0x24; }
            else if tile >= 0x11 { tile = tile.wrapping_sub(7); }
            self.tile_ram[(destination & 0x3ff) as usize] = tile;
            self.colour_ram[(destination & 0x3ff) as usize] = colour;
            source += 1;
            destination = destination.wrapping_sub(0x20);
        }
    }

    /// main:1061 c_divmod, the same original 17-pass arithmetic as sub:0EAE.
    pub fn c_divmod(value: u16, divisor: u8) -> (u16, u8) {
        divide_with_remainder(value, divisor)
    }

    /// main:0A53..0A71. The original stacks remainders, then writes them in
    /// reverse order, advancing the tile address with RST 20 each time.
    pub fn c_text_out_i_to_d(&mut self, mut value: u16, mut destination: u16) -> u16 {
        let mut digits = [0u8; 5];
        let mut count = 0;
        while value >= 10 {
            let (quotient, remainder) = Self::c_divmod(value, 10);
            digits[count] = remainder;
            count += 1;
            value = quotient;
        }
        let mut a = value as u8;
        loop {
            self.tile_ram[(destination & 0x3ff) as usize] = a;
            destination = destination.wrapping_sub(0x20);
            if count == 0 { return destination; }
            count -= 1;
            a = digits[count];
        }
    }

    /// main:0160, preserves the original final LDIR pointer overlap at 03BF.
    pub fn c_sctrl_playfld_clr(&mut self) {
        self.tile_ram[0x40..0x3c0].fill(0x24);
        self.colour_ram[0x40..0x3c0].fill(0);
        self.colour_ram[0x3bf..0x3df].fill(4);
        self.colour_ram[0x3df..0x3ff].fill(0x4e);
    }
}

fn bcd_add_one(value: u8) -> (u8, bool) {
    let (mut a, mut carry) = value.overflowing_add(1);
    let half_carry = value & 15 == 15;
    if a & 15 > 9 || half_carry { a = a.wrapping_add(6); }
    if a > 0x99 || carry { a = a.wrapping_add(0x60); carry = true; }
    (a, carry)
}
