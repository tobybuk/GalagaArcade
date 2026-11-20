//! Direct translations of the original home-coordinate and formation tasks.
//! Addresses in comments use the supplied Namco rev. B ROM, with the annotation
//! name in parentheses where the older disassembly has a different address.
use crate::rom_data::{EXPANSION_BITMAPS, FORMATION_ORIGIN};

#[derive(Clone, Debug)]
pub struct Formation {
    pub coordinates: [u16; 16],
    pub offsets: [u8; 16],
    pub direction: u8,
    current_bits: [u8; 16],
    pub sliding: bool,
    pub pulsing: bool,
}

impl Default for Formation {
    fn default() -> Self {
        // main:0x12D5 (c_12C3). Rows use the original inverted 9-bit format.
        let mut coordinates = [0; 16];
        for i in 0..10 {
            coordinates[i] = FORMATION_ORIGIN[i] as u16;
        }
        for i in 10..16 {
            coordinates[i] = (0x160 - 2 * FORMATION_ORIGIN[i] as u16) & 0x1fe;
        }
        Self { coordinates, offsets: [0; 16], direction: 0,
               current_bits: [0; 16], sliding: true, pulsing: false }
    }
}

impl Formation {
    /// main:12D5, original argument and flipped-screen branch. The column
    /// high bytes and the bitmap scratch bytes are not cleared by this call.
    pub fn initialise(&mut self, offset: u8, flip: u8) {
        self.offsets.fill(0);
        for i in 0..10 {
            let mut value = FORMATION_ORIGIN[i];
            if flip & 1 != 0 { value = !value.wrapping_add(13); }
            self.coordinates[i] = (self.coordinates[i] & 0xff00) | value as u16;
        }
        for i in 10..16 {
            let mut value = FORMATION_ORIGIN[i].wrapping_add(offset);
            if flip & 1 == 0 { value = !value.wrapping_add(0x4f); }
            self.coordinates[i] = (value as u16) << 1;
        }
        self.direction = flip;
    }

    /// main:0x2A90. Step once per four frames; reverse at offsets +32/-32.
    pub fn slide(&mut self, frame: u8, entrance_complete: bool) {
        if frame.wrapping_sub(1) & 3 != 0 { return; }
        self.sliding=true;self.pulsing=false;
        let step: u8 = if self.direction == 0 { 1 } else { 255 };
        for i in 0..10 {
            self.offsets[i] = self.offsets[i].wrapping_add(step);
            self.coordinates[i] = (self.coordinates[i]&0xff00) | (self.coordinates[i] as u8).wrapping_add(step) as u16;
        }
        if !entrance_complete || self.offsets[0] != 0 {
            match self.offsets[0] as i8 {
                32 => self.direction = 1,
                -32 => self.direction = 0,
                _ => {}
            }
        } else {
            self.direction = 0;
            self.sliding = false;
            self.pulsing = true;
        }
    }

    /// main:0x1DEC (f_1DE6). Bitmap-driven expansion, not a sine-wave substitute.
    pub fn pulse(&mut self, frame: u8) {
        if frame & 3 != 0 { return; }
        self.pulsing=true;self.sliding=false;
        let previous = self.direction;
        let step = if previous & 0x80 == 0 { 1u8 } else { 255u8 };
        self.direction = self.direction.wrapping_add(step);
        if previous == 0x1f { self.direction |= 0x80; }
        if previous == 0x81 { self.direction &= 0x7f; }
        if previous & 7 == 0 {
            let start = ((self.direction & 0x18) / 8) as usize * 16;
            self.current_bits.copy_from_slice(&EXPANSION_BITMAPS[start..start + 16]);
        }
        // main:0x1E49 (fmtn_expcon_comp).
        for i in 0..16 {
            let carry = self.current_bits[i] & 1;
            self.current_bits[i] = self.current_bits[i].rotate_right(1);
            if carry == 0 { continue; }
            let delta = if (previous & 0x80 != 0) ^ (i < 5) { 255 } else { 1 };
            self.offsets[i] = self.offsets[i].wrapping_add(delta);
            // c_1E43 also toggles bit 8 on carry/borrow. The old C aid clears
            // the high byte; retain the original nine-bit coordinate instead.
            let previous=self.coordinates[i] as u8;
            self.coordinates[i]=(self.coordinates[i]&0xff00)|previous.wrapping_add(delta) as u16;
            if (delta==1 && previous==255) || (delta==255 && previous==0) {self.coordinates[i]^=0x100;}
        }
    }
}

/// main:0x2C00. Packed stage/difficulty data, including the assembly's >=27
/// reduction and challenge-stage threshold. The old C version differs here.
pub fn stage_parameters(stage: u8, rank: u8) -> [u8; 11] {
    assert!(rank < 4);
    let mut reduced = stage;
    while reduced >= 0x1b { reduced -= 4; }
    let row = [1usize, 2, 3, 0][rank as usize];
    let decremented = reduced.wrapping_sub(1);
    let offset = decremented.rotate_left(2).wrapping_add(decremented);
    let start = row * 130 + offset as usize;
    let mut result = [0; 11];
    for i in 0..5 {
        let address = start + i;
        let packed = if address < 520 { crate::rom_data::STAGE_PARAMETERS[address] }
            else { crate::rom_data::STAGE_PARAMETER_WRAP_TAIL[address - 520] };
        result[i * 2] = packed >> 4;
        result[i * 2 + 1] = packed & 15;
    }
    result[10] = if stage < 3 || stage & 3 == 3 { 0 } else { 10 };
    result
}
