//! Direct source port of the supplied sound-board routines, sound:00BB..06D9.
//! No CPU interpreter. The RAM bytes retain original aliases and wrapping.
use crate::{rom_data::SOUND_DATA,Game};
const WAVES:&[u8;256]=include_bytes!("../assets/sound-waves.bin");
pub const SAMPLE_RATE:u32=48000;
pub const FRAME_SAMPLES:usize=792;

fn byte(address:u16)->u8 {SOUND_DATA[(address-0x06da) as usize]}
fn word(address:u16)->u16 {u16::from_le_bytes([byte(address),byte(address+1)])}

pub struct Audio {
    /// Original sound-board RAM 9A00..9AFF, including data/status/voice scratch.
    pub ram:[u8;256],
    /// Actual four-bit hardware registers; RAM keeps its original unmasked bytes.
    pub registers:[u8;32],
    pub phase:[u32;3],
    pub samples:[f32;FRAME_SAMPLES],
    pub noise:crate::noise_circuit::NoiseCircuit,
}
impl Default for Audio {
    fn default()->Self {
        let mut noise=crate::noise_circuit::NoiseCircuit::default();noise.configure_after_self_test();
        Self {ram:[0;256],registers:[0;32],phase:[0;3],samples:[0.0;FRAME_SAMPLES],noise}
    }
}

impl Game {
    /// Two sound NMIs per video period. Whole-board interleaving remains an
    /// explicit validation boundary; the routine and arbitration order are exact.
    pub fn sound_nmi(&mut self) {
        self.audio.ram[0xa0..0xc0].copy_from_slice(&self.combat.sound_requests);
        self.audio.ram[0x79]=self.coin_sound_count;
        self.audio.c_nmi_proc(self.formation_signage);
        self.combat.sound_requests.copy_from_slice(&self.audio.ram[0xa0..0xc0]);
        self.coin_sound_count=self.audio.ram[0x79];
    }
}

impl Audio {
    /// Browser hardware adaptation: the board's three 4-bit wavetable voices.
    /// Register wiring follows the documented Namco WSG. Generate at 192 kHz
    /// (16 fractional phase bits), then average four samples for 48-kHz output.
    /// This is the sound circuit, separate from the original sequencer above.
    pub fn synthesise(&mut self,start:usize,count:usize) {
        let mut frequencies=[0u32;3];let mut volumes=[0i32;3];let mut waves=[0usize;3];
        for voice in 0..3 {
            if voice==0 {frequencies[voice]=self.registers[0x10] as u32;}
            for i in 0..4 {frequencies[voice]|=(self.registers[0x11+voice*5+i] as u32)<<(4+i*4);}
            volumes[voice]=self.registers[0x15+voice*5] as i32;
            waves[voice]=(self.registers[5+voice*5]&7) as usize*32;
        }
        for sample in &mut self.samples[start..start+count] {
            let mut sum=0i32;let mut noise=0.0;
            for _ in 0..4 {
                noise+=self.noise.sample();
                for voice in 0..3 {
                    if volumes[voice]==0 {continue;}
                    let position=((self.phase[voice]>>16)&31) as usize;
                    sum+=((WAVES[waves[voice]+position]&15) as i32-8)*volumes[voice];
                    self.phase[voice]=self.phase[voice].wrapping_add(frequencies[voice]);
                }
            }
            // Board-reference WSG gain and the original explosion circuit mix.
            *sample=(sum as f64/(4.0*128.0*3.0)*(0.90*10.0/16.0)+noise/4.0).clamp(-1.0,1.0) as f32;
        }
    }
    fn load_word(&self,offset:usize)->u16 {u16::from_le_bytes([self.ram[offset],self.ram[offset+1]])}
    fn store_word(&mut self,offset:usize,value:u16) {self.ram[offset..offset+2].copy_from_slice(&value.to_le_bytes());}

    /// sound:00BB. Preserve priority, capture modulation and rev. B's jump
    /// from ship-hit music directly to the coin sound, bypassing later themes.
    pub fn c_nmi_proc(&mut self,signage:u8) {
        if self.ram[0xb8]!=0 {self.ram.fill(0);return;}
        self.ram[0x60..0x70].fill(0);
        if self.ram[0xb7]!=0 {
            self.ram[0xa0..0xb6].fill(0);self.ram[0xc0..0xd7].fill(0);
        } else {
            if self.ram[0x79]!=0 {
                self.ram[0xa8]=self.ram[0xa8].wrapping_add(self.ram[0x79]);self.ram[0x79]=0;
            }
            if self.ram[0xa0]!=0 {self.pulsing_formation(signage);}
            for sound in [0x13,0x0f,3,2,4,1] {
                self.ram[0x74]=sound;
                if self.ram[0xa0+sound as usize]!=0 {
                    self.ram[0xa0+sound as usize]=0;self.c_03f4();
                } else if self.ram[0xc0+sound as usize]!=0 {self.c_044a();}
            }
            self.request(0x12,true);
            if self.ram[0xa5]!=0 {
                self.ram[0x74]=5;self.c_0375();
                self.ram[0x7e]=self.ram[0x7e].wrapping_add(1);
                if self.ram[0x7e]>=6 {
                    self.ram[0x7e]=0;
                    self.ram[0x7c]=if self.ram[0x7c]<4 {12} else {self.ram[0x7c].wrapping_sub(1)};
                }
                self.ram[0x6f]=self.ram[0x7c];
            } else {self.ram[0xc5]=0;}
            if self.ram[0xa6]!=0 {
                self.ram[0x74]=6;self.c_0375();
                self.ram[0x7f]=self.ram[0x7f].wrapping_add(1);
                if self.ram[0x7f]==0x1c {self.ram[0x7f]=0;self.ram[0x7d]=self.ram[0x7d].wrapping_add(1);}
                // Supplied revision writes voice ZERO, not voice two in the annotation.
                self.ram[0x70]=self.ram[0x7d];
            } else {self.ram[0xc6]=0;}
            self.request(9,false);
            if self.ram[0xa7]!=0 {self.request(7,true);}
            else {
                self.request(0x11,false);self.request(0x0d,true);
                if self.ram[0xae]!=0 {
                    self.request(0x0e,true);
                    // Rev. B writes these volumes even on the note that ends the request.
                    self.ram[0x6a]=9;self.ram[0x6f]=6;
                }
                for sound in [0x14,0x15,0x0a,0x0b] {self.request(sound,true);}
                self.request(0x10,false);self.request(0x0c,true);self.request(0x16,true);
            }
            self.request(8,true);
        }
        self.j_0357_set_sfrs();
    }

    fn request(&mut self,sound:u8,consume:bool) {
        if self.ram[0xa0+sound as usize]!=0 {
            self.ram[0x74]=sound;
            if consume {self.c_04a2();} else {self.c_0375();}
        } else if !consume {self.ram[0xc0+sound as usize]=0;}
    }

    /// sound:00EB..0159 (annotation 00D3), original 34-tick frequency ramps.
    fn pulsing_formation(&mut self,signage:u8) {
        let mut update=false;
        if signage!=self.ram[0x80] {
            self.ram[0x80]=signage;
            self.store_word(0x82,if signage.wrapping_add(1)==0 {0x0704} else {0x06f4});
            self.ram[0]=0;self.ram[0x81]=0;update=true;
        } else {
            self.ram[0]=self.ram[0].wrapping_add(1);
            if self.ram[0]==0x22 {
                self.ram[0]=0;self.ram[0x81]=self.ram[0x81].wrapping_add(1);update=true;
            }
        }
        if update {
            let pointer=self.load_word(0x82).wrapping_add(self.ram[0x81] as u16*2);
            self.store_word(0x84,word(pointer));self.store_word(0x86,word(pointer+32));
        }
        self.store_word(0x86,self.load_word(0x86).wrapping_add(self.load_word(0x84)));
        let high=self.ram[0x87];self.ram[0x61]=high;self.ram[0x62]=high.rotate_right(4);
        self.ram[0x65]=10;self.ram[0x70]=0;
    }

    /// sound:0366, hardware masks nibble writes; waveform uses only bits 0..2.
    fn j_0357_set_sfrs(&mut self) {
        for i in 0..16 {self.registers[0x10+i]=self.ram[0x60+i]&15;}
        for (voice,register) in [5,10,15].into_iter().enumerate() {self.registers[register]=self.ram[0x70+voice]&15;}
    }

    fn parameters(&mut self) {
        let sound=self.ram[0x74];let source=0x0734+sound as u16*3;
        for i in 0..3 {self.ram[0x75+i]=byte(source+i as u16);}
        if sound==0x0e {
            if self.ram[0x4c]==0 {self.ram[0x76]=1;}
            else if self.ram[0x4c]==1 || self.ram[0x4d]==0 {self.ram[0x76]=2;}
        }
    }
    fn initialise_data(&mut self) {
        let start=self.ram[0x75] as usize;let count=self.ram[0x76] as usize;
        self.ram[0x30+start..0x30+start+count].fill(0);
        self.ram[start..start+count].fill(0);
    }
    fn voices(&mut self)->bool {
        loop {
            self.c_0550();self.ram[0x76]=self.ram[0x76].wrapping_sub(1);
            if self.ram[0x76]==0 {break;}
            self.ram[0x75]=self.ram[0x75].wrapping_add(1);self.ram[0x77]=self.ram[0x77].wrapping_add(1);
        }
        if self.ram[0x78]==0 {return false;}
        self.ram[0x78]=0;self.ram[0xc0+self.ram[0x74] as usize]=0;
        true
    }

    /// sound:0384 (annotation c_0375), repeat while game-controlled request is on.
    pub fn c_0375(&mut self) {
        self.parameters();let active=0xc0+self.ram[0x74] as usize;
        if self.ram[active]==0 {self.ram[active]=1;self.initialise_data();}
        self.voices();
    }
    /// sound:0403, restart effect and immediately update its voices.
    pub fn c_03f4(&mut self) {
        let active=0xc0+self.ram[0x74] as usize;self.ram[active]=self.ram[active].wrapping_add(1);
        self.parameters();self.initialise_data();self.voices();
    }
    /// sound:0459, update an already started single-shot effect.
    pub fn c_044a(&mut self) {self.parameters();self.voices();}

    /// sound:04B1, including rev. B's extra sound-seven completion branch.
    pub fn c_04a2(&mut self) {
        self.parameters();let active=0xc0+self.ram[0x74] as usize;
        if self.ram[active]==0 {self.ram[active]=1;self.initialise_data();}
        if !self.voices() {return;}
        let sound=self.ram[0x74];let request=0xa0+sound as usize;
        match sound {
            8=>self.ram[request]=self.ram[request].wrapping_sub(1),
            12=> {
                self.ram[request]=self.ram[request].wrapping_sub(1);
                if self.ram[request]==0 || self.ram[request]&1!=0 {self.ram[0xb6]=1;}
            }
            20=> {self.ram[request]=0;self.ram[0xb3]=1;}
            7=> {
                self.ram[request]=0;
                for base in [0xa0,0xc0] {
                    self.ram[base..base+8].fill(0);self.ram[base+9]=0;
                    self.ram[base+11..base+23].fill(0);
                }
            }
            _=>self.ram[request]=0,
        }
    }

    /// sound:0581..06AA. Original note lookup, octave shifts, volume envelope,
    /// eight-pass duration product and exact-equality count expiration.
    pub fn c_0550(&mut self) {
        let index=self.ram[0x75] as usize;
        self.ram[index]=self.ram[index].wrapping_add(1);
        let source=word(0x0779+index as u16*2);
        for i in 0..3 {self.ram[0x88+i]=byte(source+i as u16);}
        let source=source+3+self.ram[0x30+index] as u16;
        self.store_word(0x7a,source);
        let note=byte(source);
        if note==0xff {self.j_068b_close_voice_and_exit();return;}
        let mut frequency=word(0x06da+(note&15) as u16*2);
        for _ in 0..note>>4 {frequency>>=1;}
        let frequency_destination=match self.ram[0x77] {0=>0x61,1=>0x66,_=>0x6b};
        let [low,high]=frequency.to_le_bytes();
        self.ram[frequency_destination]=low;self.ram[frequency_destination+1]=low.rotate_right(4);
        self.ram[frequency_destination+2]=high;self.ram[frequency_destination+3]=high.rotate_right(4);
        let count=self.ram[index];
        let volume=if note==12 {0}
            else if self.ram[0x88]!=0 && count<6 {
                if self.ram[0x88]==1 {count.wrapping_add(count)} else {!count}
            } else {
                let threshold=self.ram[0x89];
                if threshold==0 || count<threshold {10}
                else {let elapsed=count.wrapping_sub(threshold);if elapsed>=10 {0} else {10u8.wrapping_sub(elapsed)}}
            };
        self.ram[frequency_destination+4]=volume;
        self.ram[0x70+self.ram[0x77] as usize]=self.ram[0x8a];
        let mut base=byte(0x07d7+self.ram[0x74] as u16);
        let mut factor=byte(source+1) as u16;let mut duration=0u16;
        for _ in 0..8 {if base&1!=0 {duration=duration.wrapping_add(factor);}base>>=1;factor=factor.wrapping_shl(1);}
        if duration as u8!=count {return;}
        self.ram[0x30+index]=self.ram[0x30+index].wrapping_add(2);self.ram[index]=0;
    }

    /// sound:06BC, original selected-volume clear and shared finished flag.
    fn j_068b_close_voice_and_exit(&mut self) {
        let destination=match self.ram[0x77] {0=>0x65,1=>0x6a,_=>0x6f};
        self.ram[destination]=0;self.ram[0x78]=1;
    }
}
