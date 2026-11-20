//! Original stage construction, class initialisation and wave-launch routines.
use crate::{rom_data::*, Game, State};

#[derive(Default)]
pub struct StageTokens {
    pub columns: Vec<(u8, bool)>,
    pub position: u8,
    pub cursor: usize,
    pub waiting: bool,
    pub target_frame: u8,
    pub delay: bool,
    pub sound: u8,
}

fn byte(address: u16) -> Result<u8, u16> {
    match address {
        0x26a8..=0x26eb => Ok(COMBAT_STAGE_INDEX[(address - 0x26a8) as usize]),
        0x26ec..=0x26f3 => Ok(CHALLENGE_STAGE_INDEX[(address - 0x26ec) as usize]),
        0x26f4..=0x27dd => Ok(COMBAT_STAGE_DATA[(address - 0x26f4) as usize]),
        0x27de..=0x286d => Ok(CHALLENGE_STAGE_DATA[(address - 0x27de) as usize]),
        0x286e..=0x2895 => Ok(WAVE_OBJECT_IDS[(address - 0x286e) as usize]),
        _ => Err(address),
    }
}

impl Game {
    pub fn is_challenge_stage(&self) -> bool { self.stage_number.wrapping_add(1) & 3 == 0 }

    /// main:0185..01C5. Continuation of the original token call/wait is kept
    /// in the background executive rather than blocking the browser thread.
    pub fn stg_init_splash(&mut self, after: crate::executive::StageAfter) {
        self.stage_number = self.stage_number.wrapping_add(1);
        let normal = self.stage_number.wrapping_add(1) & 3;
        if normal != 0 {
            let destination = self.c_string_out(6, None);
            self.c_text_out_i_to_d(self.stage_number as u16, destination);
            self.combat.flight_hits_remaining = 0;
        } else {
            self.c_string_out(7, None);
            self.combat.sound_requests[13] = 1;
            self.combat.flight_hits_remaining = 8;
        }
        self.game_timers[2] = 3;
        self.enemy_enabled = 3;
        self.c_new_level_tokens(normal, false);
        self.background = crate::executive::Background::StageTokens(after);
    }

    /// main:01C6..0236, all original stores in stage-environment order.
    pub fn stg_init_env(&mut self) {
        self.game_timers[2] = 120;
        self.initialise_enemy_classes();
        self.gctl_stg_new_atk_wavs_init();
        if self.fault.is_some() { return; }
        self.game_timers[0] = 2;
        self.formation.initialise(0, self.flip_screen);
        self.formation.sliding = true; self.formation.pulsing = false;
        self.combat.hit_notifications.fill(0);
        self.main_task_enable[9] = 0; self.main_task_enable[16] = 0; self.main_task_enable[4] = 0;
        self.combat.flying_hits = 0;
        self.boss_escort_toggle = 0; self.bonus_bee_timer = 0; self.attack_wave_enabled = 0;
        self.wave_number = 0; self.combat.squad_remaining = 0; self.nest_lr_inhibit = 0;
        self.combat.bonus_bee_id = 1; self.combat.capture_boss_id = 1;
        self.main_task_enable[11] = 1; self.main_task_enable[8] = 1; self.main_task_enable[10] = 1;
        self.c_2c00();
        self.combat.boss_score_attributes = [1, 0xb5, 1, 0xb5, 1, 0xb5, 1, 0xb5];
        self.entrance_complete = false;
    }

    /// main:2C00 and its post-unpack timer/star-speed writes.
    pub fn c_2c00(&mut self) {
        self.stage_parameters = crate::formation::stage_parameters(self.stage_number, self.machine_rank);
        self.bomber_control[..3].copy_from_slice(&[0x16, 2, 2]);
        let mut a = self.stage_number;
        if a >= 16 { a = 16; }
        self.star_control[2] = (a.rotate_left(2) & 0x70).wrapping_add(0x40);
    }

    /// main:117F..1212. The token columns retain each c_build_token_1
    /// (timed/sounded) and c_build_token_2 (immediate) call in original order.
    pub fn c_new_level_tokens(&mut self, sound: u8, inhibit_delay: bool) {
        // Supplied rev. B preserves reserve-ship tiles and redraws reserves
        // at the tail; the older annotation simply blanks both tile ranges.
        for low in (2..20).chain(0x22..0x34) {
            if self.tile_ram[low] < 0x4a { self.tile_ram[low] = 0x24; }
        }
        let mut remainder = self.stage_number;
        let mut fifties = 0u8;
        let mut position = 1u8;
        while remainder >= 50 { remainder -= 50; fifties += 1; position += 2; }
        let (tens, ones) = Self::c_divmod(remainder as u16, 10);
        let tens = tens as u8;
        let extra_ones = if ones >= 5 { ones - 4 } else { ones };
        position = position.wrapping_add(extra_ones).wrapping_add(if tens & 1 != 0 { 2 } else { tens });
        let mut columns = Vec::new();
        for _ in 0..fifties { columns.push((0x46, true)); columns.push((0x48, false)); }
        if tens != 0 {
            if tens == 4 {
                columns.push((0x42, true)); columns.push((0x44, false));
                columns.push((0x3a, true)); columns.push((0x3c, false));
            } else {
                let tile = tens.rotate_left(2).wrapping_add(0x36);
                columns.push((tile, true)); columns.push((tile.wrapping_add(2), false));
            }
        }
        let mut ones = ones;
        if ones >= 5 { columns.push((0x38, true)); ones -= 5; }
        for _ in 0..ones { columns.push((0x36, true)); }
        self.stage_tokens = StageTokens { columns, position, sound, delay: !inhibit_delay, ..StageTokens::default() };
    }

    /// main:1213..1227 c_build_token_1 wait, then its c_build_token_2 tail.
    pub fn stage_tokens_continue(&mut self) -> bool {
        while self.stage_tokens.cursor < self.stage_tokens.columns.len() {
            let (tile, timed) = self.stage_tokens.columns[self.stage_tokens.cursor];
            if timed && self.stage_tokens.delay {
                if !self.stage_tokens.waiting {
                    self.stage_tokens.waiting = true;
                    self.stage_tokens.target_frame = self.counters[0].wrapping_add(8);
                    return false;
                }
                if self.counters[0] != self.stage_tokens.target_frame { return false; }
                self.combat.sound_requests[0x15] = self.stage_tokens.sound;
            }
            self.c_build_token_2(tile);
            self.stage_tokens.waiting = false;
            self.stage_tokens.cursor += 1;
        }
        self.draw_resv_ships();
        true
    }

    /// main:1228..1241, low-byte address operations and original colour test.
    fn c_build_token_2(&mut self, mut tile: u8) {
        let low = self.stage_tokens.position;
        self.tile_ram[low as usize] = tile;
        tile = tile.wrapping_add(1);
        self.tile_ram[(low | 0x20) as usize] = tile;
        tile = tile.wrapping_add(1);
        let colour = if tile & 12 == 8 { 1 } else { 2 };
        self.colour_ram[(low | 0x20) as usize] = colour;
        self.colour_ram[(low & !0x20) as usize] = colour;
        self.stage_tokens.position = (low & !0x20).wrapping_sub(1);
    }

    /// main:1000. Original lookup/add arithmetic with the two refresh-register
    /// samples made explicit. The ROM page is read as data, never instructions.
    pub fn c_1000_samples(&self, first: u8, second: u8) -> u8 {
        let index = self.counters[0].wrapping_add(first);
        REFRESH_SAMPLE_PAGE[index as usize].wrapping_add(second)
    }

    fn refresh_advance(&mut self, fetches: u8) {
        self.refresh_counter = (self.refresh_counter & 0x80) | (self.refresh_counter.wrapping_add(fetches) & 0x7f);
    }

    /// Refresh sampling within this direct routine follows its original fetch
    /// spacing. The entry phase supplied by the host is still provisional.
    fn c_1000(&mut self) -> u8 {
        self.refresh_advance(4); // CALL, PUSH HL, ED-prefixed LD A,R
        let first = self.refresh_counter;
        self.refresh_advance(9);
        let second = self.refresh_counter;
        self.refresh_advance(3); // ADD, POP HL, RET
        self.c_1000_samples(first, second)
    }

    /// The divider's original branch-dependent fetch count matters to the
    /// next refresh sample. This is timing metadata, not opcode execution.
    fn c_divmod_refresh(&mut self, mut quotient: u16, divisor: u8) -> (u16, u8) {
        let mut remainder = 0u8;
        let mut carry = 0u16;
        self.refresh_advance(6); // initial four instructions and final POP/RET
        for _ in 0..17 {
            let doubled = remainder as u16 * 2 + carry;
            remainder = doubled as u8;
            let subtract = doubled & 0x100 != 0 || remainder >= divisor;
            self.refresh_advance(if doubled & 0x100 == 0 && subtract { 9 } else { 8 });
            let bit = if subtract { remainder = remainder.wrapping_sub(divisor); 1 } else { 0 };
            let shifted = quotient as u32 * 2 + bit;
            quotient = shifted as u16;
            carry = ((shifted >> 16) & 1) as u16;
        }
        (quotient, remainder)
    }

    /// main:25A2..26A7. Preserve temporary sixteen-byte wave slots, insertion
    /// order, rejected random positions, left/right pairing and rogue tail.
    pub fn gctl_stg_new_atk_wavs_init(&mut self) {
        if let Err(address) = self.c_25a2() { self.fault = Some(address); }
    }

    fn c_25a2(&mut self) -> Result<(), u16> {
        let original_stage = self.stage_number;
        let mut stage = original_stage;
        while stage >= 0x17 { stage = stage.wrapping_sub(4); }
        let (index, data) = if stage.wrapping_add(1) & 3 != 0 {
            let row = Self::c_104e_mul_16_8(17, self.machine_rank) as u8;
            let column = stage.wrapping_sub(stage >> 2).wrapping_sub(1);
            (0x26a8u16 + row as u16 + column as u16, 0x26f4u16)
        } else { (0x26ecu16 + ((original_stage >> 2) & 7) as u16, 0x27deu16) };
        let mut source = data + byte(index)? as u16;
        self.bomb_reload = byte(source)?;
        self.bomb_enable_flags = byte(source + 1)?;
        source += 2;
        let mut destination = 0usize;
        let mut object_source = 0usize;
        self.wave_sequence[destination] = 0x7e;
        destination += 1;
        loop {
            let mut transient = byte(source)?;
            source += 1;
            if transient == 0xff { break; }
            let mut temporary = [0xffu8; 16];
            let mut count = transient & 15;
            if count != 0 {
                let divisor = (count >> 1).wrapping_add(4);
                while count != 0 {
                    let random = self.c_1000();
                    self.refresh_advance(4); // LD L,A / LD H,0 / LD A,E / CALL
                    let (_, mut position) = self.c_divmod_refresh(random as u16, divisor);
                    self.refresh_advance(8 + if count & 1 != 0 { 2 } else { 0 });
                    if count & 1 != 0 { position |= 8; }
                    if temporary[position as usize] != 0xff { continue; }
                    let mut id = count.rotate_left(1);
                    let carry = transient & 0x80 != 0;
                    transient = transient.rotate_left(1);
                    self.refresh_advance(8 + u8::from(carry));
                    if carry { id |= 0x40; }
                    temporary[position as usize] = id | 0x38;
                    count -= 1;
                }
            }
            let mut position = 0usize;
            for remaining in (1..=8).rev() {
                while temporary[position] != 0xff { position += 1; }
                temporary[position] = WAVE_OBJECT_IDS[object_source];
                object_source += 1;
                position += 1;
                if remaining == 5 { position = 8; }
            }
            let left = byte(source)?;
            let right = byte(source + 1)?;
            source += 2;
            position = 0;
            loop {
                self.wave_sequence[destination] = left;
                if temporary[position] == 0xff { break; }
                self.wave_sequence[destination + 1] = temporary[position];
                self.wave_sequence[destination + 2] = right;
                self.wave_sequence[destination + 3] = temporary[position | 8];
                destination += 4;
                position += 1;
            }
            self.wave_sequence[destination] = 0x7e;
            destination += 1;
        }
        destination -= 1;
        if self.two_fighters.wrapping_sub(1) & self.combat.capture_boss_flag != 0 && !self.is_challenge_stage() {
            self.wave_sequence[destination] = self.wave_sequence[destination - 4];
            self.wave_sequence[destination + 1] = 4;
            destination += 2;
            self.objects[2].code = 0x87;
        }
        self.wave_sequence[destination] = 0x7f;
        Ok(())
    }

    /// main:2896..28E8, normal/challenge class and hit-eight attributes.
    pub fn initialise_enemy_classes(&mut self) {
        self.wave_cursor = 0;
        let (bee, moth) = if self.is_challenge_stage() {
            let c = self.stage_number.rotate_right(2);
            let b = c.rotate_right(1);
            let index = if b & 0x1c == 0 { b & 3 } else { 3 } as usize;
            self.combat.challenge_attributes.copy_from_slice(&CHALLENGE_HIT_ATTRIBUTES[index * 2..index * 2 + 2]);
            let packed = CHALLENGE_CLASSES[(c & 7) as usize];
            (packed, packed)
        } else { (0x36, 0x24) };
        let mut remaining = 1u8;
        let mut object = 8u8;
        let mut mask_index = 0usize;
        let mut mask = 0u8;
        self.atkw_eclass_init(&mut remaining, &mut object, &mut mask_index, &mut mask, 20, bee);
        self.atkw_eclass_init(&mut remaining, &mut object, &mut mask_index, &mut mask, 8, 0x10);
        self.atkw_eclass_init(&mut remaining, &mut object, &mut mask_index, &mut mask, 16, moth);
    }

    /// main:28E9: reload, RLC C, RRA A, then advance the original object ID.
    fn atkw_eclass_init(&mut self, remaining: &mut u8, object: &mut u8,
                       mask_index: &mut usize, mask: &mut u8, mut count: u8, packed: u8) {
        while count != 0 {
            *remaining = remaining.wrapping_sub(1);
            if *remaining == 0 { *mask = BOMB_CLASS_MASKS[*mask_index]; *mask_index += 1; *remaining = 8; }
            let carry = *mask >> 7;
            *mask = mask.rotate_left(1);
            self.objects[*object as usize / 2].code = (packed >> 1) | (carry << 7);
            *object = object.wrapping_add(2);
            count -= 1;
        }
    }

    /// main:2916..2A3B, original markers, challenge pacing, transient remap,
    /// packed vector table and writes into an available original motion slot.
    pub fn wave_task(&mut self) {
        let token = self.wave_sequence[self.wave_cursor];
        if token == 0x7f {
            if self.flying_count != 0 { return; }
            self.entrance_complete = true;
            self.main_task_enable[8] = 0; self.main_task_enable[4] = 1; self.main_task_enable[16] = 1;
            self.nest_lr_inhibit = 1;
            return;
        }
        if token == 0x7e {
            if self.attack_wave_enabled == 0 { return; }
            if self.flying_count != 0 { self.game_timers[0] = 2; return; }
            if self.is_challenge_stage() {
                if self.game_timers[0] == 1 { self.combat.flight_hits_remaining = 8; return; }
                if self.game_timers[0] != 0 { return; }
            }
            self.wave_cursor += 1;
            self.wave_number = self.wave_number.wrapping_add(1);
            return;
        }
        if token & 0x80 == 0 && self.counters[0] & 7 != 0 { return; }
        let shifted = token << 1;
        let Some(slot) = self.motions.iter().position(|m| !m.active()) else { return; };
        let original_id = self.wave_sequence[self.wave_cursor + 1];
        let id = if original_id & 0x78 == 0x78 { original_id & !0x40 } else { original_id };
        self.wave_cursor += 2;
        let object = &mut self.objects[id as usize / 2];
        object.state = State::Spawning; object.motion_slot = slot as u8;
        let motion = &mut self.motions[slot];
        motion.object_id = id;
        if id & 0x38 != 0x38 {
            let packed = object.code;
            object.code = packed & 0x78; object.colour = packed & 7;
            motion.bomb_flags = if packed & 0x80 == 0 { 0 } else { self.bomb_enable_flags };
        } else {
            let (code, colour) = if original_id & 0x40 != 0 { (0x10, 2) }
                else if self.wave_number == 2 { (8, 0) } else { (0x18, 3) };
            object.code = code; object.colour = colour; motion.bomb_flags = 0;
        }
        motion.bomb_counter = if shifted & 2 == 0 { 8 } else { 0x44 };
        let index = (shifted & 0x7f) as usize;
        let packed = u16::from_le_bytes([ENTRANCE_VECTORS[index], ENTRANCE_VECTORS[index + 1]]);
        motion.path = packed & 0x1fff;
        let initial = (((packed >> 12) as usize) & 14) * 3 + if shifted & 0x80 == 0 { 0 } else { 3 };
        motion.y = (ENTRANCE_INITIALS[initial] as u16) << 8;
        motion.x = (ENTRANCE_INITIALS[initial + 1] as u16) << 8;
        motion.angle = (ENTRANCE_INITIALS[initial + 2] as u16) << 8;
        motion.duration = 1;
        motion.flags = (shifted | 1) & 0x81;
    }

    /// main:1A86..1B6A, including the original CPI scan, colour flashing,
    /// launch recheck and three-attacker score/bitmap configuration.
    pub fn f_1a80(&mut self) {
        if self.active_enemy_count >= self.stage_parameters[10] { return; }
        if self.bonus_bee_timer == 0 {
            let candidate = (4..24).chain(32..48).find(|&id| self.objects[id].state == State::Standby);
            let Some(id) = candidate else { return; };
            self.bonus_bee_timer = 0xc0;
            self.bonus_bee_class = self.objects[id].colour;
            let (_, remainder) = Self::c_divmod((self.stage_number >> 2) as u16, 3);
            self.bonus_bee_colour = remainder.wrapping_add(4);
            self.combat.bonus_bee_id = (id * 2) as u8;
            self.combat.sound_requests[0x12] = self.bonus_bee_colour;
            return;
        }
        let next = self.bonus_bee_timer.wrapping_add(1);
        let id = self.combat.bonus_bee_id as usize / 2;
        if next != 0 {
            self.bonus_bee_timer = next;
            if self.objects[id].state != State::Standby { self.main_task_enable[4] = 0; return; }
            self.objects[id].colour = if next & 16 == 0 { self.bonus_bee_class } else { self.bonus_bee_colour };
            return;
        }
        if self.main_task_enable[21] == 0 { self.bonus_bee_timer = 0xe0; return; }
        if self.objects[id].state != State::Standby || self.hit_notification_byte(id) & 0x80 != 0 {
            self.main_task_enable[4] = 0; return;
        }
        let index = self.bonus_bee_colour.wrapping_sub(4) as usize;
        self.combat.squad_remaining = 3;
        self.combat.squad_bonus_increment = BONUS_BEE_ATTRIBUTES[index * 2];
        self.combat.squad_bonus_sprite = BONUS_BEE_ATTRIBUTES[index * 2 + 1];
        let path = u16::from_le_bytes([BONUS_BEE_VECTORS[index * 2], BONUS_BEE_VECTORS[index * 2 + 1]]);
        let original_code = self.objects[id].code;
        self.objects[id].code = (index as u8).rotate_left(3).wrapping_add(0x56);
        self.bonus_bee_class = (original_code & 0xf8) | (self.bonus_bee_class & 7);
        self.c_1083(self.combat.bonus_bee_id, path);
        self.main_task_enable[4] = 0;
    }
}
