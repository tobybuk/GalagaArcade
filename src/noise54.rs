//! Source-level reconstruction of the supplied 54xx firmware, not a CPU model.
//! Original nibble RAM addresses identify the recovered variables and branches.
//! Circuit timing/output integration is separate in noise_circuit.rs.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ParameterTransfer {pub destination:usize,pub remaining:u8}

pub struct Noise54 {
    pub ram:[u8;64],
    pub output:[u8;3],
    pub parameters:Option<ParameterTransfer>,
    pub iterations:u64,
}
impl Default for Noise54 {
    fn default()->Self {Self {ram:[0;64],output:[0;3],parameters:None,iterations:0}}
}

fn add_nibbles(left:u8,right:u8,carry:bool)->(u8,bool) {
    let sum=left+right+u8::from(carry);(sum&15,sum&16!=0)
}

impl Noise54 {
    /// 54xx:0006..0011 clears all nibble RAM, then enables external commands.
    pub fn reset(&mut self) {*self=Self {ram:[0;64],output:[0;3],parameters:None,iterations:0};}

    /// 54xx:0002/JPA0180, original command jump table and parameter transfer.
    /// Commands are latched by the external 06xx, never fetched as instructions.
    pub fn command(&mut self,command:u8) {
        if let Some(mut transfer)=self.parameters {
            self.ram[transfer.destination]=command&15;
            self.ram[transfer.destination-1]=command>>4;
            transfer.remaining-=1;
            if transfer.remaining==0 {self.parameters=None;}
            else {transfer.destination-=2;self.parameters=Some(transfer);}
            return;
        }
        match command>>4 {
            1=>self.trigger_a_01c0(),
            2=>self.trigger_b_01dc(),
            3=>self.parameters=Some(ParameterTransfer {destination:0x37,remaining:4}),
            4=>self.parameters=Some(ParameterTransfer {destination:0x3f,remaining:4}),
            5=>self.trigger_c_01f9(),
            6=>self.parameters=Some(ParameterTransfer {destination:0x29,remaining:5}),
            7=>self.ram[0x2f]=command&15, // 0168, direct volume command
            _=>{}, // original 01BC no-operation return
        }
    }

    /// 54xx:01C0, original type-A initial volume/state and high counter bytes.
    pub fn trigger_a_01c0(&mut self) {
        self.ram[0x15]=self.ram[0x30];self.ram[0x18]=3;
        self.ram[0x17]=self.ram[0x37];self.ram[0x16]=self.ram[0x36];
    }
    /// 54xx:01DC, original type-B initial volume/state and high counter bytes.
    pub fn trigger_b_01dc(&mut self) {
        self.ram[0x1a]=self.ram[0x38];self.ram[0x1d]=3;
        self.ram[0x1c]=self.ram[0x3f];self.ram[0x1b]=self.ram[0x3e];
        self.ram[1]=self.ram[0x3f]; // original XD 1 scratch write
    }
    /// 54xx:01F9, original type-C 12-bit counter, volume and two-stage envelope.
    pub fn trigger_c_01f9(&mut self) {
        self.ram[0x2c]=self.ram[0x21];self.ram[0x2d]=self.ram[0x22];
        self.ram[0x2e]=self.ram[0x23];self.ram[0x2f]=self.ram[0x20];self.ram[0x1f]=2;
    }

    /// 54xx:0012, shift the same four nibbles, then compute and store the fifth.
    /// The first ROR discards its result, making its incoming carry irrelevant.
    pub fn noise_a_0012(&mut self)->u16 {
        let mut carry=self.ram[0x14]&1;
        for address in (0x10..0x14).rev() {
            let previous=self.ram[address];
            self.ram[address]=(previous>>1)|(carry<<3);carry=previous&1;
        }
        let rotated=(self.ram[0x10]>>1)|(carry<<3);
        self.ram[0x14]=((rotated^self.ram[0x13])+1)&15;
        let enabled=self.ram[0x10]&1!=0;
        self.output[0]=if enabled {self.ram[0x15]} else {0};
        // 002F..0040 also feeds the second noise source's four-bit divider.
        self.ram[0x19]=(self.ram[0x19]*2+u8::from(enabled))&15;
        if enabled {38} else {36}
    }

    fn decrement(&mut self,address:usize)->bool {
        let previous=self.ram[address];self.ram[address]=previous.wrapping_sub(1)&15;
        previous==0 // original DCM status permits the next borrow only on zero
    }

    /// 54xx:0041..008F, including its zero-duration phase skipping loop.
    pub fn envelope_a_0041(&mut self)->u16 {
        let mut cycles=4;
        if self.ram[0x18]==0 {return cycles;}
        cycles+=4;if !self.decrement(8) {return cycles;}
        cycles+=5;if !self.decrement(0x16) {return cycles;}
        cycles+=4;if !self.decrement(0x17) {return cycles;}
        cycles+=1;
        loop {
            self.decrement(0x18);
            let selector=if self.ram[0x18]==1 {
                cycles+=12;
                self.ram[0x15]=self.ram[0x31];1
            } else if self.ram[0x18]==2 {
                cycles+=11;
                self.ram[0x15]=0;2
            } else {
                cycles+=11;
                if self.ram[0x15]==0 {return cycles;}
                cycles+=12;
                // 0084..008A: ROR, RSTC, ADC, ROR. Preserve nibble carries.
                let half=self.ram[0x15]>>1;
                let (sum,carry)=add_nibbles(half,self.ram[0x15],false);
                self.ram[0x15]=(sum>>1)|(u8::from(carry)<<3);
                self.ram[0x18]=1;1
            };
            let parameter=0x30+selector*2;
            let low=self.ram[parameter];let high=self.ram[parameter+1];
            self.ram[0]=high; // final XD 0 retains the high parameter nibble
            if low==0 && high==0 {cycles+=15;continue;}
            cycles+=if low!=0 {17} else {19};
            self.ram[0x16]=low;self.ram[0x17]=high;return cycles;
        }
    }

    /// 54xx:0090..00A6, original second divider, one-bit gate and type-B volume.
    pub fn noise_b_0090(&mut self)->u16 {
        let mut cycles=6;
        if self.ram[0x19]&3==1 {self.ram[0x1e]=self.ram[0x1e].wrapping_add(1)&15;cycles+=2;}
        let muted=self.ram[0x1e]&1!=0;
        self.output[1]=if muted {0} else {self.ram[0x1a]};
        cycles+if muted {13} else {11}
    }

    /// 54xx:00A7..00FB, same explicit type-B branch structure as its firmware.
    pub fn envelope_b_00a7(&mut self)->u16 {
        let mut cycles=5;
        if self.ram[0x1d]==0 {return cycles;}
        cycles+=4;if !self.decrement(9) {return cycles;}
        cycles+=5;if !self.decrement(0x1b) {return cycles;}
        cycles+=4;if !self.decrement(0x1c) {return cycles;}
        cycles+=1;
        loop {
            self.decrement(0x1d);
            let selector=if self.ram[0x1d]==1 {
                cycles+=13;
                self.ram[0x1a]=self.ram[0x39];1
            } else if self.ram[0x1d]==2 {
                cycles+=14;
                self.ram[0x1a]=0;2
            } else {
                cycles+=12;
                if self.ram[0x1a]==0 {return cycles;}
                cycles+=12;
                let half=self.ram[0x1a]>>1;
                let (sum,carry)=add_nibbles(half,self.ram[0x1a],false);
                self.ram[0x1a]=(sum>>1)|(u8::from(carry)<<3);
                self.ram[0x1d]=1;1
            };
            let parameter=0x38+selector*2;
            let low=self.ram[parameter];let high=self.ram[parameter+1];
            self.ram[0]=high;
            if low==0 && high==0 {cycles+=19;continue;}
            cycles+=if low!=0 {18} else {21};
            self.ram[0x1b]=low;self.ram[0x1c]=high;return cycles;
        }
    }

    /// 54xx:00FC..0120, type-C gate followed by both original carry additions.
    pub fn tone_c_00fc(&mut self)->u16 {
        let muted=self.ram[0x2b]&8!=0;
        let mut cycles=if muted {24} else {23};
        self.output[2]=if muted {0} else {self.ram[0x2f]};
        let (low,carry)=add_nibbles(self.ram[0x28],self.ram[0x2a],false);
        self.ram[0x2a]=low;
        self.ram[0x2b]=add_nibbles(self.ram[0x29],self.ram[0x2b],carry).0;
        if self.ram[0x10]&1==0 {return cycles;}
        cycles+=13;
        let (low,carry)=add_nibbles(self.ram[0x26],self.ram[0x2a],false);
        self.ram[0x2a]=low;
        self.ram[0x2b]=add_nibbles(self.ram[0x27],self.ram[0x2b],carry).0;
        cycles
    }

    /// 54xx:0121..0167, original counter underflows and diminishing-volume path.
    pub fn envelope_c_0121(&mut self)->u16 {
        let mut cycles=5;
        if self.ram[0x1f]==0 {return cycles;}
        cycles+=5;if !self.decrement(0x2c) {return cycles;}
        cycles+=4;if !self.decrement(0x2d) {return cycles;}
        cycles+=4;if !self.decrement(0x2e) {return cycles;}
        cycles+=7;
        self.decrement(0x1f);
        if self.ram[0x1f]==0 {
            cycles+=10;
            let volume=self.ram[0x2f];
            let half=volume>>1;
            let (sum,carry)=add_nibbles(half,volume,false);
            self.ram[0x2f]=(sum>>1)|(u8::from(carry)<<3);
            if self.ram[0x2f]==0 {return cycles;}
            cycles+=3;
            self.ram[0x1f]=1;
        }
        self.ram[0x2d]=self.ram[0x24];self.ram[0x2e]=self.ram[0x25];
        if self.ram[0x24]==0 && self.ram[0x25]==0 {
            cycles+=19;
            self.ram[0x2f]=0;self.ram[0x1f]=0;return cycles;
        }
        self.ram[0x2c]=0;
        cycles+if self.ram[0x24]==0 {18} else {16}
    }

    /// Original main loop order. Timing integration will schedule the output
    /// boundaries above, rather than replace this firmware with a host RNG.
    pub fn iteration(&mut self)->bool {
        if self.parameters.is_some() {return false;} // firmware waits inside parameter IRQ
        self.noise_a_0012();self.envelope_a_0041();
        self.noise_b_0090();self.envelope_b_00a7();
        self.tone_c_00fc();self.envelope_c_0121();
        self.iterations=self.iterations.wrapping_add(1);true
    }
}
