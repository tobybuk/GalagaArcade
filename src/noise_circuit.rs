//! Timing and analogue-output adaptation around the translated 54xx firmware.
//! Circuit component values/equations follow MAME's BSD-3-Clause galaga_a.cpp,
//! disc_flt.hxx and disc_mth.hxx (Derrick Renaud / K. Wilkins; see NOTICE.md).
use std::collections::VecDeque;
use crate::{noise54::Noise54,rom_data::{NOISE_INITIAL_PARAMETERS,NOISE_TRIGGER_COMMANDS}};

const RATE:f64=192000.0;
const VREF:f64=2.0;
const RESISTORS:[f64;4]=[47000.0,22000.0,10000.0,4700.0];

struct Bandpass {r1:f64,total:f64,a1:f64,a2:f64,b0:f64,b2:f64,x1:f64,x2:f64,y1:f64,y2:f64}
impl Bandpass {
    fn new(series:f64,bias:f64,feedback:f64,cap:f64)->Self {
        let dac_resistance=1.0/RESISTORS.iter().map(|r|1.0/r).sum::<f64>();
        let r1=dac_resistance+series;
        let total=1.0/(1.0/r1+1.0/bias);
        let frequency=1.0/(2.0*std::f64::consts::PI*(total*feedback*cap*cap).sqrt());
        let damping=(cap+cap)/(feedback/total*cap*cap).sqrt();
        let gain=-feedback/total/2.0;
        let w=RATE*2.0*(std::f64::consts::PI*frequency/RATE).tan();
        let t=2.0*RATE;let denominator=t*t+damping*w*t+w*w;
        let b0=damping*w*t/denominator*gain;
        // Start after the board self-test, with the un-driven filter at rest.
        let input=-VREF/r1*total;
        Self {r1,total,a1:2.0*(-t*t+w*w)/denominator,
            a2:(t*t-damping*w*t+w*w)/denominator,b0,b2:-b0,x1:input,x2:input,y1:0.0,y2:0.0}
    }
    fn sample(&mut self,digital:u8)->f64 {
        let mut current=0.0;let mut conductance=0.0;
        for (bit,resistor) in RESISTORS.iter().enumerate() {
            current+=if digital&(1<<bit)!=0 {4.0/resistor} else {0.0};
            conductance+=1.0/resistor;
        }
        let dac=current/conductance;
        let input=(dac-VREF)/self.r1*self.total;
        let voltage=(-self.a1*self.y1-self.a2*self.y2+self.b0*input+self.b2*self.x2+VREF).clamp(0.0,3.5);
        self.x2=self.x1;self.x1=input;self.y2=self.y1;self.y1=voltage-VREF;
        voltage
    }
}

/// These six phases are the recovered firmware sections, not CPU instructions.
#[derive(Clone,Copy)]
enum Section {NoiseA,EnvelopeA,NoiseB,EnvelopeB,ToneC,EnvelopeC}

pub struct NoiseCircuit {
    pub firmware:Noise54,
    pub ticks:u64,
    section:Section,
    section_tick:u64,
    pending_output:Option<(u64,usize,u8)>,
    dac:[u8;3],
    transfers:VecDeque<(u64,u8)>,
    filters:[Bandpass;3],
    coupling_cap:f64,
    coupling_coefficient:f64,
}
impl Default for NoiseCircuit {
    fn default()->Self {
        Self {firmware:Noise54::default(),ticks:0,section:Section::NoiseA,section_tick:0,
            pending_output:None,dac:[0;3],transfers:VecDeque::new(),
            filters:[Bandpass::new(100000.0,22000.0,220000.0,1e-9),
                Bandpass::new(47000.0,10000.0,150000.0,1e-8),
                Bandpass::new(150000.0,22000.0,470000.0,1e-8)],
            coupling_cap:0.0,coupling_coefficient:1.0-(-1.0/(RATE*100000.0*1e-7)).exp()}
    }
}
impl NoiseCircuit {
    pub fn configure_after_self_test(&mut self) {
        for &command in &NOISE_INITIAL_PARAMETERS {self.firmware.command(command);}
    }
    /// main:0066 NMI's A8 command: four transfers at 48kHz/32 = 1500Hz.
    /// The global 06xx read/write phase remains a validation boundary.
    pub fn trigger_bang(&mut self) {
        self.command_packet(&NOISE_TRIGGER_COMMANDS);
    }
    pub fn command_packet(&mut self,commands:&[u8]) {
        let mut at=self.transfers.back().map_or(self.ticks+16,|(tick,_)|*tick+512);
        for &command in commands {self.transfers.push_back((at,command));at+=512;}
    }
    fn next_section(&mut self) {
        use Section::*;
        let at=self.section_tick;
        let (cycles,next)=match self.section {
            NoiseA=> {
                let cycles=self.firmware.noise_a_0012();
                let offset=if self.firmware.ram[0x10]&1!=0 {29} else {32};
                self.pending_output=Some((at+offset*3,0,self.firmware.output[0]));
                (cycles,EnvelopeA)
            }
            EnvelopeA=>(self.firmware.envelope_a_0041(),NoiseB),
            NoiseB=> {
                let increment=self.firmware.ram[0x19]&3==1;
                let cycles=self.firmware.noise_b_0090();
                let offset=13+u64::from(increment)*2+u64::from(self.firmware.ram[0x1e]&1!=0);
                self.pending_output=Some((at+offset*3,1,self.firmware.output[1]));
                (cycles,EnvelopeB)
            }
            EnvelopeB=>(self.firmware.envelope_b_00a7(),ToneC),
            ToneC=> {
                let muted=self.firmware.ram[0x2b]&8!=0;
                let cycles=self.firmware.tone_c_00fc();
                self.pending_output=Some((at+(8+u64::from(muted))*3,2,self.firmware.output[2]));
                (cycles,EnvelopeC)
            }
            EnvelopeC=> {
                let cycles=self.firmware.envelope_c_0121();
                self.firmware.iterations=self.firmware.iterations.wrapping_add(1);(cycles,NoiseA)
            }
        };
        self.section=next;self.section_tick+=cycles as u64*3;
    }
    /// 768-kHz shared tick unit: one MB8844 instruction cycle is three ticks,
    /// one 192-kHz internal audio sample is four ticks. No opcode execution.
    pub fn sample(&mut self)->f64 {
        loop {
            let transfer_tick=self.transfers.front().map_or(u64::MAX,|(at,_)|*at);
            let output_tick=self.pending_output.map_or(u64::MAX,|(at,_,_)|at);
            let section_tick=if self.firmware.parameters.is_some() {u64::MAX} else {self.section_tick};
            let event=transfer_tick.min(output_tick).min(section_tick);
            if event>self.ticks {break;}
            if event==transfer_tick {
                let (_,command)=self.transfers.pop_front().unwrap();
                self.firmware.command(command);
                // IRQ dispatch and the original handler hold the main loop.
                let cycles=match command>>4 {1=>37,2=>38,_=>12};
                self.section_tick=self.section_tick.max(event)+cycles*3;
                if let Some((at,channel,value))=self.pending_output {
                    self.pending_output=Some((at+cycles*3,channel,value));
                }
            } else if event==output_tick {
                let (_,channel,value)=self.pending_output.take().unwrap();self.dac[channel]=value;
            } else {self.next_section();}
        }
        self.ticks+=4;
        // Galaga routes outputs 2,1,0 through three different band-pass filters.
        let voltages=[self.filters[0].sample(self.dac[2]),self.filters[1].sample(self.dac[1]),self.filters[2].sample(self.dac[0])];
        let voltage=((VREF-voltages[0])/33000.0+(VREF-voltages[1])/33000.0+(VREF-voltages[2])/10000.0)*3300.0;
        self.coupling_cap+=(voltage-self.coupling_cap)*self.coupling_coefficient;
        (voltage-self.coupling_cap)*40800.0/32768.0*0.90
    }
}
