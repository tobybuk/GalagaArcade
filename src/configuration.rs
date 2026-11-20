//! Original service configuration decoder/display, main:37FE..3961.
//! The two physical DIP banks are presented through the board's bit wiring.
use crate::{rom_data::CONFIGURATION_DATA,Game};
fn byte(address:u16)->u8 {CONFIGURATION_DATA[(address-0x3a72) as usize]}
fn word(address:u16)->u16 {u16::from_le_bytes([byte(address),byte(address+1)])}

impl Game {
    /// Hardware adaptation: each original read exposes one bit from each bank.
    pub fn configure_machine(&mut self,dswa:u8,dswb:u8) {
        for i in 0..8 {self.dip_ports[i]=((dswb>>i)&1)|(((dswa>>i)&1)<<1);}
        self.unused_dsw3=self.dip_ports[2];self.attract_sound_dsw=self.dip_ports[3];
        self.machine_dsw5=self.dip_ports[4];self.machine_dsw6=self.dip_ports[5];
        self.c_svc_updt_dsply();
    }
    fn three_dips(&self,start:usize)->u8 {
        let mut value=0u8;
        for i in start..start+3 {value=value.wrapping_shl(1)|(self.dip_ports[i]&1);}
        value&7
    }
    /// main:37FE. Same cabinet, rank, lives, default-controller parameters,
    /// coinage and bonus branches/calls as the original configuration screen.
    pub fn c_svc_updt_dsply(&mut self) {
        self.machine_cabinet=((self.dip_ports[7]>>1).wrapping_add(1))&1;
        self.c_svc_cab_type(0x3ad6+self.machine_cabinet as u16*2);
        self.flip_screen=u8::from(self.io_input[0]&12==0);
        self.machine_rank=((self.dip_ports[0]>>1)&1)|(self.dip_ports[1]&2);
        self.tile_ram[0x22c]=byte(0x3a72+self.machine_rank as u16);
        self.c_text_out(0x3aee);
        self.machine_reserves=((self.dip_ports[6].wrapping_shl(1)|(self.dip_ports[7]&1))&3).wrapping_add(1);
        self.tile_ram[0x2ea]=self.machine_reserves.wrapping_add(1);
        self.c_text_out(0x3af5); // rev. B: FIGHTERS, rather than older SHIPS string
        for i in 0..8 {self.io_parameters[i]=byte(0x3ace+i as u16);}
        let selection=self.three_dips(0);
        if selection!=0 {
            let mut source=0x3a76+selection.wrapping_sub(1) as u16*8;
            for i in 1..5 {self.io_parameters[i]=byte(source);source+=1;}
            for address in [0x2e8,0x228,0x1e8,0xe8] {self.tile_ram[address]=byte(source);source+=1;}
            self.tile_ram[0x208]=0x24;
            let source=self.c_text_out(0x3b00);self.c_text_out(source);
        } else {
            self.io_parameters[1..5].fill(0);self.c_text_out(0x3b11);
        }
        let selection=self.three_dips(3);
        if selection==0 {
            self.c_text_out(0x3b71);self.machine_bonus=[0xff,0xff];
            self.l_393b();return;
        }
        let index=((self.machine_reserves&4).wrapping_shl(1)).wrapping_add(selection).wrapping_shl(1);
        let mut source=0x3aae+index as u16;
        self.machine_bonus=[byte(source),byte(source+1)];source+=1;
        self.c_38da(source,1);source-=1;self.c_38da(source,0);
    }

    /// main:3A6B, original pointer dereference followed by c_text_out.
    fn c_svc_cab_type(&mut self,source:u16) {self.c_text_out(word(source));}

    /// main:38E4, original first/second/every branch and helper call order.
    fn c_38da(&mut self,source:u16,pass:u8) {
        let value=byte(source);
        if value==0xff {self.l_393b();return;}
        let text=word(0x3b27+pass as u16*2);
        let text=self.c_text_out(text);self.c_text_out(text);
        self.c_391e(0x81f0+if pass==1 {2} else {0},value&0x7f);
        if pass.wrapping_sub(1)!=0 {return;}
        if value&0x80!=0 {self.l_3949();return;}
        self.c_391e(0x81f4,value&0x7f);
        let text=self.c_text_out(0x3b5a);self.c_text_out(text);
    }
    /// main:3928, two stores, with RES 5,L rather than generic text formatting.
    fn c_391e(&mut self,address:u16,mut value:u8) {
        let mut tens=0x24;
        if value>=10 {tens=1;value=value.wrapping_sub(10);}
        self.tile_ram[(address&0x3ff) as usize]=tens;
        self.tile_ram[((address&!0x20)&0x3ff) as usize]=value;
    }
    fn l_393b(&mut self) {self.erase_bonus_line(0x8332);self.l_3949();}
    fn l_3949(&mut self) {self.erase_bonus_line(0x8334);}
    fn erase_bonus_line(&mut self,mut address:u16) {
        for _ in 0..22 {self.tile_ram[(address&0x3ff) as usize]=0x24;address=address.wrapping_sub(32);}
    }
}
