//! Direct source reconstruction of the supplied Namco 51xx controller firmware.
//! Addresses name the original routines and nibble RAM, not CPU instructions.
//! The external board adapter supplies four active-low physical input nibbles.

pub struct Input51 {
    pub ram:[u8;128],
    pub output:u8,
    pub lockout:u8,
    pub timer:u8,
    parameters:Option<u8>,
}

impl Default for Input51 {
    fn default()->Self {
        let mut result=Self {ram:[0;128],output:0,lockout:0,timer:0,parameters:None};
        result.reset_007e();result
    }
}

impl Input51 {
    /// 007E..0095: RAM clear, initial output/unused input bytes and switch mode.
    pub fn reset_007e(&mut self) {
        self.ram.fill(0);self.ram[0x1f]=12;
        self.ram[0x2a]=15;self.ram[0x2b]=15;self.ram[3]=1;
        self.lockout=0;self.timer=0;self.parameters=None;
    }

    /// 0002, table 0040..007D, parameter receiver 0008..0021 and return 0022.
    /// Only K0..K2 carry command/parameter data on this board.
    pub fn command(&mut self,data:u8) {
        let data=data&7;
        if let Some(destination)=self.parameters {
            if data==0 {self.ram[0x1a]=10;}
            self.ram[destination as usize]=data;
            if destination==0x10 {self.parameters=None;}
            else {self.parameters=Some(destination-1);return;}
        } else {
            match data {
                1=>{self.parameters=Some(0x13);return;},
                2=>self.ram[3]=0,
                3=>self.ram[2]=1,
                4=>self.ram[2]=0,
                5=>self.ram[3]=1,
                _=>{}, // original no-operation command table entries
            }
        }
        self.ram[6]=4;
    }

    /// 024F..0264: two original OUTO writes and the four-byte read cursor.
    pub fn read_024f(&mut self)->u8 {
        let cursor=self.ram[6] as usize;
        let result=self.ram[0x20+cursor]|(self.ram[0x21+cursor]<<4);
        let next=(cursor as u8+2)&15;
        self.ram[6]=if next==12 {4} else {next};
        result
    }

    /// 06xx read stretch primes the first byte before the first host NMI.
    /// Three host reads each also select the MCU: the fourth result is unused.
    pub fn transfer_three(&mut self)->[u8;3] {
        let result=[self.read_024f(),self.read_024f(),self.read_024f()];
        self.read_024f();result
    }

    /// Main loop 0097..0186, run on each change of the external video counter.
    pub fn video_sample_0097(&mut self,ports:[u8;4]) {
        self.timer=self.timer.wrapping_add(1);self.ram[9]=self.timer&15;
        self.ram[0x18]=0;
        if self.ram[3]==1 {
            self.ram[0x24]=ports[2]&15;self.ram[0x25]=ports[3]&15;
            self.ram[0x27]=ports[1]&15;self.ram[0x26]=ports[0]&15;
            self.ram[0x28..0x2c].fill(15);self.lockout=0;return;
        }
        if ports[3]&8==0 {self.test_switch_0239();return;}
        if self.ram[0x1a]!=10 {
            self.filter_coins_00cb(ports[3]&15);
            if self.ram[0x14]&1==0 {self.coin_a_0187();}
            if self.ram[0x14]&2==0 {self.coin_b_01a4();}
            if self.ram[0x14]&4==0 {self.service_credit_01c6();}
        }
        self.publish_0265();
        self.filter_buttons_00ec(ports[2]&15);
        if self.ram[3]!=0 {self.controls_01f9(ports);}
        else if self.ram[0x18]==0 {
            if self.ram[0x1a]!=0 || self.ram[0x19]>1 {
                if self.ram[0x23]&4==0 {self.start_01d9(1);}
                else if self.ram[0x23]&8==0 {self.start_01d9(2);}
            } else if self.ram[0x19]==1 && self.ram[0x23]&4==0 {
                self.start_01d9(1);
            }
        }
        self.outputs_0120();
    }

    /// 00CB..00D9: three exchanges, AND/NEG/AI F and two ORs.
    fn filter_coins_00cb(&mut self,input:u8) {
        let a=self.ram[0x15];let b=self.ram[0x16];let c=self.ram[0x17];
        self.ram[0x15]=input;self.ram[0x16]=a;self.ram[0x17]=b;
        let mut filtered=(c&b).wrapping_neg()&15;
        filtered=filtered.wrapping_add(15)&15;
        filtered|=a;filtered|=input;self.ram[0x14]=filtered;
    }
    /// 00EC..00F9: same original filter, with the reverse RAM direction.
    fn filter_buttons_00ec(&mut self,input:u8) {
        let a=self.ram[0x22];let b=self.ram[0x21];let c=self.ram[0x20];
        self.ram[0x22]=input;self.ram[0x21]=a;self.ram[0x20]=b;
        let mut filtered=(c&b).wrapping_neg()&15;
        filtered=filtered.wrapping_add(15)&15;
        filtered|=a;filtered|=input;self.ram[0x23]=filtered;
    }

    pub fn coin_a_0187(&mut self) {
        self.ram[0x1d]=(self.ram[0x1d]+1)&15;
        if self.ram[0x1d]==self.ram[0x13] {
            self.ram[0x1d]=0;let increment=self.ram[0x12];
            self.ram[0x18]=increment;self.add_credit_digits(increment);
        }
        self.ram[0x1b]=(self.ram[0x1b]+1).min(15);
    }
    pub fn coin_b_01a4(&mut self) {
        self.ram[0x1e]=(self.ram[0x1e]+1)&15;
        if self.ram[0x1e]==self.ram[0x11] {
            self.ram[0x1e]=0;let increment=self.ram[0x10];
            self.ram[0x18]|=increment;self.add_credit_digits(increment);
        }
        self.ram[0x1c]=(self.ram[0x1c]+1).min(15);
    }
    pub fn service_credit_01c6(&mut self) {
        self.ram[0x18]=1;self.add_credit_digits(1);
    }
    fn add_credit_digits(&mut self,increment:u8) {
        // Retain the controller's nibble ADC/DAA, including DAA carry output.
        let sum=self.ram[0x19]+increment;
        let mut digit=sum&15;
        if sum&16!=0 || digit>9 {digit+=6;}
        self.ram[0x19]=digit&15;
        self.ram[0x1a]=(self.ram[0x1a]+u8::from(digit&16!=0))&15;
        if self.ram[0x1a]==10 {self.ram[0x1a]=9;self.ram[0x19]=9;}
    }
    /// 01D7..01F7: subtract, publish, disable starts, clear fractions/free play.
    pub fn start_01d9(&mut self,cost:u8) {
        let borrow=self.ram[0x19]<cost;
        let mut digit=self.ram[0x19].wrapping_sub(cost)&15;
        if borrow || digit>9 {digit+=10;}
        self.ram[0x19]=digit&15;
        self.ram[0x1a]=self.ram[0x1a].wrapping_sub(u8::from(digit&16!=0))&15;
        self.publish_0265();self.ram[3]=2;
        self.ram[0x1d]=0;self.ram[0x1e]=0;
        if self.ram[0x13]==0 {self.ram[0x19]=0;self.ram[0x1a]=10;}
    }

    /// 01F9..0238, both independent panel bytes, with optional 0272 remapping.
    fn controls_01f9(&mut self,ports:[u8;4]) {
        self.ram[0x27]=(self.ram[0x23]&1)|((self.ram[0x22]&1)<<1);
        self.ram[0x29]=((self.ram[0x23]&2)>>1)|(self.ram[0x22]&2);
        self.ram[0x26]=if self.ram[2]==0 {Self::remap_0272(ports[0])} else {ports[0]&15};
        self.ram[0x28]=if self.ram[2]==0 {Self::remap_0272(ports[1])} else {ports[1]&15};
    }
    pub fn remap_0272(input:u8)->u8 {
        [15,14,13,5,12,9,7,6,11,3,10,4,1,2,0,8][(input&15) as usize]
    }
    fn test_switch_0239(&mut self) {
        self.ram[0x19]=0;self.ram[0x1a]=0;self.lockout=0;
        self.ram[0x1d]=0;self.ram[0x1e]=0;
        self.ram[0x24]=11;self.ram[0x25]=11;self.ram[3]=1;self.output=15;
    }
    fn publish_0265(&mut self) {
        self.ram[0x24]=self.ram[0x19];self.ram[0x25]=self.ram[0x1a];
    }
    /// 0120..017D, queued coin-meter pulses, start lamps and serial lockout.
    fn outputs_0120(&mut self) {
        self.ram[0x1f]&=12;
        if self.ram[0x1b]!=0 {
            if self.timer&15==4 {
                if self.ram[0x1f]&8==0 {self.ram[0x1f]|=8;self.ram[0x1b]-=1;}
            } else if self.timer&15==0 {self.ram[0x1f]&=!8;}
        }
        if self.ram[0x1c]!=0 {
            if self.timer&15==12 {
                if self.ram[0x1f]&4==0 {self.ram[0x1f]|=4;self.ram[0x1c]-=1;}
            } else if self.timer&15==8 {self.ram[0x1f]&=!4;}
        }
        if self.ram[3]==0 && (self.ram[0x1a]!=0 || self.ram[0x19]!=0) {
            if (self.ram[0x1a]!=0 || self.ram[0x19]>1) && self.timer&16!=0 {
                self.ram[0x1f]=(self.ram[0x1f]+1)&15;
            }
            if self.timer&16!=0 {self.ram[0x1f]=(self.ram[0x1f]+2)&15;}
        }
        self.output=self.ram[0x1f];self.lockout=if self.ram[0x1a]==0 {15} else {0};
    }
}
