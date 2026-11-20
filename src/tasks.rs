//! Original task-vector dispatch. These are function calls into translated
//! routines, not instruction decoding. A missing routine is an explicit stop.
use crate::{rom_data::*, Game, State};

fn vector(table: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([table[index * 2], table[index * 2 + 1]])
}

impl Game {
    /// main interrupt kernel: the enable byte is saved BEFORE the call. Both
    /// assembly kernels do this; the older C aid re-reads the main task byte.
    pub fn cpu0_rst38(&mut self) {
        if !self.main_irq_enabled() {return;}
        if !self.attract_setup_pending {
            let c=self.counters[0]&0x1c;
            let speed=if self.machine_dsw5&2!=0 {self.star_control[5]&7} else {7};
            self.star_port_latch=speed|((c.rotate_right(1)^c)&0x18);
        }
        self.main_task_trace_len = 0;
        if self.machine_dsw5&2==0 {return;}
        let mut index = if self.attract_setup_pending {2} else {0};
        while index < 32 {
            let advance = self.main_task_enable[index];
            if advance == 0 { index += 1; continue; }
            let address = vector(&MAIN_TASK_VECTORS, index);
            self.main_task_trace[self.main_task_trace_len] = address;
            self.main_task_trace_len += 1;
            match address {
                0x0827 => {},
                0x0828 => self.f_0828(),
                0x17b2 => self.f_17b2(),
                0x1700 => self.f_1700(),
                0x0977 => self.f_0977(),
                0x1d7c => self.f_1d76(),
                0x1d38 => self.f_1d32(),
                0x0857 => self.f_0857(),
                0x2916 => self.wave_task(),
                0x1dec => {
                    if self.counters[0]&3==0 {self.formation_signage=if self.formation.direction&0x80!=0 {0xff} else {1};}
                    self.formation.pulse(self.counters[0]);
                },
                0x2a90 => self.f_2a90(),
                0x1db9 => self.f_1db3(),
                0x23dd => self.object_task(),
                0x1eaa => self.f_1ea4(),
                0x1b6b => self.f_1b65(),
                0x2000 => self.f_2000(),
                0x20f2 => self.f_20f2(),
                0x21cb => self.f_21cb(),
                0x2222 => self.f_2222(),
                0x19b2 => self.f_19b2(),
                0x0935 => self.f_0935(),
                0x1a86 => self.f_1a80(),
                0x1f8f => self.fighter_control(self.io_input[1 + (self.flip_screen & 1) as usize]),
                0x1f0e => { if self.io_input[1 + (self.flip_screen & 1) as usize] & 0x10 == 0 { self.launch_rocket(); } },
                0x1dd8 => self.f_1dd2(),
                _ => self.fault = Some(address),
            }
            if self.attract_setup_pending {return;}
            if address==0x0977 && self.service_active {return;}
            if self.fault.is_some() { return; }
            index = (index + advance as usize) & 255;
        }
    }

    /// sub:0513 periodic executive, including exact frame-divider sequence.
    pub fn cpu1_rst38(&mut self) {
        if self.machine_dsw5&2==0 {return;}
        self.counters[0] = self.counters[0].wrapping_add(1);
        if self.counters[0] & 0x1f == 1 {
            self.counters[2] = self.counters[2].wrapping_add(1);
        } else if self.counters[0] & 0x1f == 0 {
            self.counters[2] |= 1;
            self.counters[1] = self.counters[1].wrapping_add(1);
            self.counters[2] = self.counters[2].wrapping_add(1);
        }
        self.continuous_bombing = u8::from(self.active_enemy_count < self.stage_parameters[7])
            & self.main_task_enable[0x15] & 1;
        let mut index = 0usize;
        while index < 8 {
            let advance = self.sub_task_enable[index];
            if advance == 0 { index += 1; continue; }
            let address = vector(&SUB_TASK_VECTORS, index);
            match address {
                0x05be => {},
                0x05bf => self.f_05bf(),
                0x08d3 => self.f_08d3(),
                0x06f5 => self.rocket_motion(),
                0x05ee => self.f_05ee(),
                0x0eca => self.f_0eca(),
                _ => self.fault = Some(address),
            }
            if self.fault.is_some() { return; }
            index = (index + advance as usize) & 255;
        }
    }

    /// sub:0ECA..0EDD. On the supplied board 10DF/10FF are outside the
    /// populated 4K ROM. Two reads of the same static location XOR to zero,
    /// so the unused switch takes RST 00, rather than the external-editor tail.
    pub fn f_0eca(&mut self) {
        if self.unused_dsw3&2!=0 {return;}
        let _unused_10ff=0xffu8;
        let previous_10df=0xffu8;let current_10df=0xffu8;
        if (current_10df^previous_10df)&16==0 {self.sub_reset_057c();}
    }

    /// sub:057C..05B0 after the ROM checksum/ready handshake. The importer
    /// verifies that supplied ROM; no CPU or unmapped-ROM execution is needed.
    pub fn sub_reset_057c(&mut self) {
        self.sub_diagnostic[0]=0;
        self.sub_task_enable[1..].copy_from_slice(&[1,1,0,1,1,0,10]);
    }

    /// sub:0EDE..0FD4, original optional external memory-editor tail. The
    /// supplied board's static 10DF reads take reset before this entry. A data
    /// read/write callback models that editor's target, never CPU execution.
    pub fn sub_editor_0ede(&mut self,ports:[u8;5],memory:&mut impl FnMut(u16,Option<u8>)->u8) {
        self.sub_diagnostic.copy_within(4..23,5); // original nineteen-byte LDDR
        for (group,value) in ports.into_iter().enumerate() {self.sub_diagnostic[4+group*4]=value;}
        let mut selected=None;
        for group in 0..5 {
            let i=4+group*4;
            let mask=!(self.sub_diagnostic[i]|self.sub_diagnostic[i+1])&
                self.sub_diagnostic[i+2]&self.sub_diagnostic[i+3]&15;
            if mask!=0 {selected=Some((group,mask));break;}
        }
        let mut write=false;
        if let Some((group,mut mask))=selected {
            if group==4 {
                if mask&1!=0 {self.sub_diagnostic[0]=5;}
                else if self.sub_diagnostic[0]>>1==0 {
                    let address=u16::from_le_bytes([self.sub_diagnostic[2],self.sub_diagnostic[3]]);
                    let address=if mask&8!=0 {address.wrapping_add(1)} else {address.wrapping_sub(1)};
                    self.sub_diagnostic[2..4].copy_from_slice(&address.to_le_bytes());self.sub_diagnostic[0]=1;
                } else if mask&8!=0 {self.sub_diagnostic[0]=self.sub_diagnostic[0].wrapping_sub(1);}
                else if self.sub_diagnostic[0]<5 {self.sub_diagnostic[0]+=1;}
                else {self.sub_diagnostic[0]=5;}
            } else {
                let mut digit=(3-group) as u8*4;
                loop {
                    let carry=mask&1!=0;mask=mask.rotate_right(1);
                    if carry {break;}
                    digit=digit.wrapping_add(1);
                }
                let selection=self.sub_diagnostic[0];let target=(selection>>1) as usize+1;
                let mut value=self.sub_diagnostic[target];
                if selection&1!=0 {value=value.rotate_left(4);}
                value=(value&0xf0)|digit;
                if selection&1!=0 {value=value.rotate_left(4);}
                self.sub_diagnostic[target]=value;
                self.sub_diagnostic[0]=if selection==0 {1} else {selection.wrapping_sub(1)};
                write=selection>>1==0;
            }
        }
        let address=u16::from_le_bytes([self.sub_diagnostic[2],self.sub_diagnostic[3]]);
        if write {memory(address,Some(self.sub_diagnostic[1]));}
        else {self.sub_diagnostic[1]=memory(address,None);}
        let mut destination=0x83ca;
        for byte in 0..3 {self.sub_hex_0fc6(self.sub_diagnostic[1+byte],&mut destination);}
        let mut selection=self.sub_diagnostic[0];
        for digit in 0..6 {
            self.colour_ram[0x3ca+digit]=u8::from(selection!=0);selection=selection.wrapping_sub(1);
        }
    }

    /// sub:0FC6..0FD4, original nibble order and two low-byte-only increments.
    fn sub_hex_0fc6(&mut self,value:u8,destination:&mut u16) {
        self.tile_ram[(*destination&0x3ff) as usize]=value&15;
        *destination=(*destination&0xff00)|((*destination as u8).wrapping_add(1) as u16);
        self.tile_ram[(*destination&0x3ff) as usize]=value.rotate_left(4)&15;
        *destination=(*destination&0xff00)|((*destination as u8).wrapping_add(1) as u16);
    }

    /// main:0828 and sub:05BF copy separate 64-byte halves in each of the
    /// three original sprite banks. Sequential execution removes only the
    /// hardware handshake busy-waits, not the copy locations or call order.
    pub fn f_0828(&mut self) { self.sprite_ram[32..64].copy_from_slice(&self.objects[32..64]); }
    pub fn f_05bf(&mut self) { self.sprite_ram[..32].copy_from_slice(&self.objects[..32]); }

    /// main:0857, c_08AD and c_08BE. Tables are contiguous original ROM data.
    pub fn f_0857(&mut self) {
        let time = self.game_timers[2];
        if time < 0x3c { self.stage_parameters[4] = self.stage_parameters[5]; }
        let selection = self.active_enemy_count as usize / 10;
        self.bomber_control[8] = BOMBER_TIMER_READS[60 + self.stage_parameters[0] as usize * 4 + selection];
        if self.continuous_bombing != 0 {
            self.bomber_control[4..7].fill(2);
            self.combat.sound_requests[0] = 0;
            return;
        }
        self.bomber_control[4] = BOMBER_TIMER_READS[92 + self.stage_parameters[1] as usize * 4 + selection];
        let column = usize::from(time < 0x28) + usize::from(time == 0);
        self.bomber_control[5] = BOMBER_TIMER_READS[self.stage_parameters[2] as usize * 3 + column];
        self.bomber_control[6] = BOMBER_TIMER_READS[30 + self.stage_parameters[3] as usize * 3 + column];
    }

    /// main:2A90. Preserve the task-enable transitions in addition to the
    /// recovered arithmetic. Task 9 can only run on a subsequent scheduler pass.
    pub fn f_2a90(&mut self) {
        if self.counters[0].wrapping_sub(1) & 3 != 0 { return; }
        if self.active_enemy_count == 0 && self.main_task_enable[8] == 0 {
            self.main_task_enable[10] = 0;
            self.formation.sliding = false;
            return;
        }
        self.formation.slide(self.counters[0], self.nest_lr_inhibit != 0);
        if !self.formation.sliding {
            self.main_task_enable[10] = 0;
            self.main_task_enable[9] = 1;
            self.combat.sound_requests[0] = 1;
        }
    }

    /// main:1DD8, f_1DD2 in the older annotation. This tests the divided-clock
    /// bit each call exactly as the original does, rather than a wall-clock timer.
    pub fn f_1dd2(&mut self) {
        if self.counters[2] & 1 != 0 { return; }
        for timer in &mut self.game_timers { if *timer != 0 { *timer -= 1; } }
    }

    /// main:1EAA, f_1EA4 in the older annotation. Preserve fixed point residuals
    /// and the alternating two/three-pixel vertical increment.
    pub fn f_1ea4(&mut self) {
        let mut dy = (self.counters[0] & 1) as u16 + 2;
        if self.flip_screen != 0 { dy = dy.wrapping_neg(); }
        for n in 0..8 {
            let object = &mut self.objects[52 + n];
            if object.code != 0x30 || object.x == 0 {
                // rev. B:1F05 explicitly releases the object. The older
                // annotated version only skips it, which changes the flow.
                object.state = State::Inactive;
                continue;
            }
            let [rate, residual] = self.bomb_rates[n];
            let sum = residual.wrapping_add(rate & 0x7e);
            self.bomb_rates[n][1] = sum & 0x1f;
            let dx = if rate & 0x80 != 0 { (sum >> 5).wrapping_neg() } else { sum >> 5 };
            object.x = object.x.wrapping_add(dx);
            object.y = object.y.wrapping_add(dy) & 0x1ff;
        }
    }
}
