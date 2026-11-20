//! Browser video-hardware adaptation of the original Namco 05xx circuit.
//! Based on Robert Hildinger's hardware reverse engineering in MAME's
//! starfield_05xx.cpp (BSD-3-Clause; see NOTICE.md). No gameplay lives here.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Star {pub x:u16,pub y:u8,pub colour:u8}
pub struct Starfield {pub lfsr:u16,pub visible:Vec<Star>}
impl Default for Starfield {fn default()->Self {Self {lfsr:0x7fff,visible:Vec::new()}}}
fn next(value:u16)->u16 {
    let bit=value^(value>>3)^(value>>5)^(value>>10);
    (value>>1)|((bit&1)<<15)
}
impl Starfield {
    pub fn reset(&mut self) {self.lfsr=0x7fff;self.visible.clear();}
    /// Advance on every board video period, independently of browser redraws.
    /// Galaga ties vertical-scroll pins low; raw coordinates include X offset 16.
    pub fn frame(&mut self,ports:u8) {
        self.visible.clear();
        let speed=[0i32,1,2,3,-4,-3,-2,-1][(ports&7) as usize];
        for _ in 0..22*256+speed {self.lfsr=next(self.lfsr);}
        let set_a=(ports>>3)&1;let set_b=((ports>>4)&1)|2;
        for y in 0..224 {
            for x in 16..272 {
                let value=self.lfsr;
                if value&0xfa14==0x7800 {
                    let set=(((value>>10)&1)<<1)|((value>>8)&1);
                    if set==set_a as u16 || set==set_b as u16 {
                        let colour=!(((value>>5)&7)|((value<<3)&0x18)|((value<<2)&0x20))&0x3f;
                        self.visible.push(Star {x,y,colour:colour as u8});
                    }
                }
                self.lfsr=next(value);
            }
        }
        for _ in 0..10*256 {self.lfsr=next(self.lfsr);}
    }
}
