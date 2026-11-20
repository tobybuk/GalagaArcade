//! Original main:3000..336B high-score/initials and length-encoded text.
use crate::{executive::Background, rom_data::*, Game};

fn rom_byte(address: u16) -> u8 {
    match address {
        0x327f..=0x32ec => TOP5_TEXT_LAYOUT[(address - 0x327f) as usize],
        0x3345..=0x336b => HEROES_TEXT[(address - 0x3345) as usize],
        0x3197..=0x31a0 => TOP5_HIGHLIGHT_ROWS[(address - 0x3197) as usize],
        0x31a1..=0x31a5 => TOP5_NAME_SHIFT_LENGTHS[(address - 0x31a1) as usize],
        0x31ed..=0x31f6 => TOP5_SCORE_POINTERS[(address - 0x31ed) as usize],
        0x3a72..=0x3b9e => CONFIGURATION_DATA[(address-0x3a72) as usize],
        0x3a51..=0x3a6a => SERVICE_SOUND_DATA[(address-0x3a51) as usize],
        _ => panic!("Unmapped high-score data {address:04x}"),
    }
}
fn rom_word(address: u16) -> u16 { u16::from_le_bytes([rom_byte(address),rom_byte(address+1)]) }

impl Game {
    /// main:0317/032A, the supplied ROM spells N.N / A.A / M.M / C.C / O.O.
    pub fn initialise_top5(&mut self) {
        for rank in 0..5 {
            self.top5_ram[0x20+rank*6..0x26+rank*6].copy_from_slice(&HIGH_SCORE_INITIAL);
            let c = TOP5_INITIAL_NAME_CHARS[rank];
            self.top5_ram[0x3e+rank*3..0x41+rank*3].copy_from_slice(&[c,0x2a,c]);
        }
    }

    fn top5_word(&self, low: usize) -> u16 { u16::from_le_bytes([self.top5_ram[low],self.top5_ram[low+1]]) }
    fn top5_store_word(&mut self, low: usize, value: u16) { self.top5_ram[low..low+2].copy_from_slice(&value.to_le_bytes()); }
    fn score_read(&self, address: u16) -> u8 {
        if address & 0xff00 == 0x8a00 { self.top5_ram[(address & 255) as usize] }
        else { self.tile_ram[(address & 0x3ff) as usize] }
    }
    fn score_lddr(&mut self, source: &mut u16, destination: &mut u16, count: u8) {
        for _ in 0..count {
            self.top5_ram[(*destination & 255) as usize] = self.score_read(*source);
            *source = source.wrapping_sub(1); *destination = destination.wrapping_sub(1);
        }
    }

    /// main:31F7, spaces and ties retain the original comparison/carry rules.
    pub fn c_31f7_chk_score_rank(&self, mut table: u16) -> bool {
        let mut player = self.top5_word(0);
        for _ in 0..6 {
            let mut a = self.score_read(table);
            let digit = self.score_read(player);
            if a == 0x24 {
                if a == digit { player -= 1; table -= 1; continue; }
                a = 0;
            } else if digit == 0x24 { return false; }
            if a != digit { return a < digit; }
            player -= 1; table -= 1;
        }
        false
    }

    /// main:3000, five comparisons, original tune requests, score/name shifts
    /// and setup of the initials screen before its two-second wait.
    pub fn c_top5_dlg_proc(&mut self) {
        self.top5_store_word(0, if self.player_number == 0 { 0x83fd } else { 0x83e8 });
        if !self.c_31f7_chk_score_rank(0x8a3d) { self.top5_return(); return; }
        let mut rank = 5u8;
        for (table, placement) in [(0x8a37,4),(0x8a31,3),(0x8a2b,2),(0x8a25,1)] {
            if !self.c_31f7_chk_score_rank(table) { break; }
            rank = placement;
        }
        if rank == 1 { self.combat.sound_requests[12] = 0xff; }
        else { self.combat.sound_requests[16] = rank; }
        self.top5_ram[0x11] = rank;
        self.c_3118_insert_top5_score();
        let count = TOP5_NAME_SHIFT_LENGTHS[(rank-1) as usize];
        let mut source = 0x8a49; let mut destination = 0x8a4c;
        if count != 0 { self.score_lddr(&mut source, &mut destination, count); }
        self.top5_store_word(4,source);
        for _ in 0..3 { source += 1; self.top5_ram[(source & 255) as usize] = 0x24; }
        self.top5_ram[0x10] = 0x49;
        let source = self.c_text_out_ce(0x327f);
        let source = self.c_text_out(source);
        self.c_text_out_ce(source);
        self.c_3275(self.top5_word(0),0x8309);
        for address in [0x149,0x129,0x109] { self.tile_ram[address]=10; }
        self.c_puts_top5scores(); self.c_plyr_initials_entry_hilite_line();
        self.game_timers[2] = 4;
        self.background = Background::Top5IntroWait;
    }

    /// main:3118 -> rank-specific 31B0/31B4/31B8/31CE/31D9 cases.
    fn c_3118_insert_top5_score(&mut self) {
        let rank = self.top5_ram[0x11];
        match rank {
            1..=3 => {
                let a = match rank { 1 => 0x12, 2 => 0x0c, _ => 6 };
                let mut source = 0x8a37; let mut destination = 0x8a3d;
                self.score_lddr(&mut source,&mut destination,6);
                destination=0x8a37;
                self.score_lddr(&mut source,&mut destination,a);
            }
            4 => { let mut source=0x8a37; let mut destination=0x8a3d; self.score_lddr(&mut source,&mut destination,6); }
            _ => {}
        }
        let mut source=self.top5_word(0);
        let mut destination=rom_word(0x31ed+(rank as u16-1)*2);
        self.score_lddr(&mut source,&mut destination,6);
    }

    /// main:30B5, redraw and retain the original frame-sync register C.
    pub fn l_30b5_next_char_selectn(&mut self) {
        self.c_puts_top5scores(); self.c_plyr_initials_entry_hilite_line();
        self.top5_previous_frame=self.counters[0];
        self.background=Background::Top5Entry;
    }

    pub fn top5_entry_frame(&mut self) {
        if self.c_32ed_top5_dlg_endproc() { return; }
        let frame=self.counters[0];
        if frame==self.top5_previous_frame { return; }
        self.top5_previous_frame=frame;
        if frame&15==0 { self.c_3141_xor_char_color(); }
        let input=self.io_input[if self.flip_screen==0 { 1 } else { 2 }];
        if input&16==0 { self.j_314c_select_char(); return; }
        let controls=input&10;
        if controls!=self.top5_ram[2] { self.top5_ram[2]=controls; self.top5_ram[3]=0xfd; }
        self.top5_ram[3]=self.top5_ram[3].wrapping_add(1);
        if self.top5_ram[3]&15!=0 { return; }
        let address=0x100+self.top5_ram[0x10] as usize;
        let a=self.tile_ram[address];
        if controls==8 {
            self.game_timers[2]=40;
            let mut a=a.wrapping_add(1);
            if a==0x2b { a=10; }
            if a==0x25 { a=0x2a; }
            self.tile_ram[address]=a;
        } else if controls==2 {
            self.game_timers[2]=40;
            let mut a=a.wrapping_sub(1);
            if a==9 { a=0x2a; }
            if a==0x29 { a=0x24; }
            self.tile_ram[address]=a;
        }
    }

    fn c_3141_xor_char_color(&mut self) { self.colour_ram[0x100+self.top5_ram[0x10] as usize]^=5; }

    /// main:314C: commit one character, or reset the shared frame counter to
    /// 76 and wait for wrap after the third, as the original does.
    fn j_314c_select_char(&mut self) {
        let address=0x100+self.top5_ram[0x10] as usize;
        self.colour_ram[address]=0;
        let c=self.tile_ram[address];
        self.game_timers[2]=40;
        let destination=self.top5_word(4).wrapping_add(1);
        self.top5_ram[(destination&255) as usize]=c;
        self.top5_store_word(4,destination);
        let (next,carry)=self.top5_ram[0x10].overflowing_sub(32);
        self.top5_ram[0x10]=next;
        if !carry { self.l_30b5_next_char_selectn(); return; }
        self.c_puts_top5scores(); self.c_plyr_initials_entry_hilite_line();
        self.counters[0]=76;
        self.background=Background::Top5FinishWait;
    }

    /// main:32ED, timeout/coin-in commits whichever input characters remain.
    fn c_32ed_top5_dlg_endproc(&mut self) -> bool {
        let credits=self.io_input[0];
        if (credits==0xa0 || self.credit_count>=credits) && self.game_timers[2]!=0 { return false; }
        let mut source=0x8100|self.top5_ram[0x10] as u16;
        let mut destination=self.top5_word(4).wrapping_add(1);
        loop {
            self.top5_ram[(destination&255) as usize]=self.tile_ram[(source&0x3ff) as usize];
            destination=destination.wrapping_add(1);
            source=source.wrapping_sub(32);
            if source&0x0100==0 { break; }
        }
        self.top5_return(); true
    }

    /// Return from c_top5_dlg_proc into the game-over executive's sound wait.
    pub fn top5_return(&mut self) { self.combat.sound_requests[16]=0; self.background=Background::GameOverSoundWait; }

    fn c_plyr_initials_entry_hilite_line(&mut self) {
        let rank=self.top5_ram[0x11];
        let mut address=rom_word(0x3197+(rank as u16-1)*2);
        for _ in 0..22 { self.colour_ram[(address&0x3ff) as usize]=5; address=address.wrapping_sub(32); }
    }

    pub fn c_mach_hiscore_show(&mut self) {
        let source=self.c_text_out_ce(0x3345); self.c_text_out_ce(source);
        self.c_puts_top5scores();
    }

    fn c_puts_top5scores(&mut self) {
        self.c_text_out(0x32b4);
        for b in 1..=5 { self.c_3231(b); }
    }

    fn c_3231(&mut self,b:u8) {
        let mut source=0x32c5+(b as u16-1)*8;
        let mut destination=rom_word(source); source+=2;
        self.tile_ram[(destination&0x3ff) as usize]=b;
        destination=destination.wrapping_sub(32);
        for _ in 0..2 {
            self.tile_ram[(destination&0x3ff) as usize]=rom_byte(source);
            source+=1; destination=destination.wrapping_sub(32);
        }
        destination=destination.wrapping_sub(64);
        destination=self.c_3275(rom_word(source),destination); source+=2;
        destination=destination.wrapping_sub(192);
        let mut name=rom_word(source);
        for _ in 0..3 {
            self.tile_ram[(destination&0x3ff) as usize]=self.score_read(name);
            name+=1; destination=destination.wrapping_sub(32);
        }
    }

    fn c_3275(&mut self,mut source:u16,mut destination:u16) -> u16 {
        for _ in 0..6 {
            self.tile_ram[(destination&0x3ff) as usize]=self.score_read(source);
            source=source.wrapping_sub(1); destination=destination.wrapping_sub(32);
        }
        destination
    }

    /// main:331B, length encoded text without colour writes.
    pub(crate) fn c_text_out(&mut self,mut source:u16) -> u16 {
        let mut destination=rom_word(source); source+=2;
        let count=rom_byte(source); source+=1;
        for _ in 0..count {
            self.tile_ram[(destination&0x3ff) as usize]=rom_byte(source);
            source+=1; destination=destination.wrapping_sub(32);
        }
        source
    }

    /// main:3328, the original explicit length and colour-encoded variant.
    fn c_text_out_ce(&mut self,mut source:u16) -> u16 {
        let mut destination=rom_word(source); source+=2;
        let count=rom_byte(source); source+=1;
        let colour=rom_byte(source); source+=1;
        for _ in 0..count {
            self.tile_ram[(destination&0x3ff) as usize]=rom_byte(source);
            self.colour_ram[(destination&0x3ff) as usize]=colour;
            source+=1; destination=destination.wrapping_sub(32);
        }
        source
    }
}
