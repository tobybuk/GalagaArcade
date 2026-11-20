//! Original service executive, main:35BF..37F5 and helpers 3962..3A6A.
//! Physical destructive RAM/checksum work at 336C..35BE is represented by Rust
//! allocation and the checked importer. Its successful display/timing route,
//! post-test menu and controls remain here.
use crate::{executive::Background,rom_data::*,Game};

impl Game {
    pub(crate) fn main_irq_enabled(&self)->bool {
        !matches!(self.background,Background::ServiceLeaveWrap|
            Background::ServiceConfigure|Background::ServiceConfigureWait)
    }

    /// 336C hardware self-test entry, continuing at 35BF after valid resources.
    /// Machine totals at 99E0..99FF survive the original RAM-test save/restore.
    pub fn service_enter_336c(&mut self) {
        self.service_active=true;self.attract_setup_pending=false;
        self.service_input.fill(0);self.shared_9260.fill(0);
        self.combat.sound_requests.fill(0);self.audio.ram.fill(0);
        self.audio.registers.fill(0);self.audio.noise=Default::default();
        self.input51.reset_007e();
        self.io_input.fill(0);self.credit_count=0;
        self.objects.fill(Default::default());self.sprite_ram.fill(Default::default());
        self.motions.fill(Default::default());
        self.c_tileram_regs_clr_3962();self.c_spriteposn_regs_init_397c();
        self.c_text_out(0x3b8b); // supplied rev. B RAM OK
        self.c_text_out(0x3b95); // 35BF ROM OK
        self.c_svc_updt_dsply();
        self.main_task_enable.fill(0);self.main_task_enable[0]=0x20;
        self.sub_reset_057c();self.sub_task_enable[0]=7;
        for _ in 0..4 {self.input51.command(5);}
        self.counters[0]=0;self.background=Background::ServiceWaitTwo;
    }

    pub(crate) fn service_continue(&mut self) {
        use Background::*;
        match self.background {
            ServiceWaitTwo=>{
                if self.counters[0]!=2 {return;}
                // Original 12-byte A8 packet: both parameter blocks, then A/B.
                let mut commands=[0;12];commands[..10].copy_from_slice(&NOISE_INITIAL_PARAMETERS);
                commands[10]=0x10;commands[11]=0x20;
                self.audio.noise.command_packet(&commands);
                self.c_svc_test_sound_sel_39fc();self.counters[0]=0;
                self.background=ServiceWaitEight;
            }
            ServiceWaitEight=>{
                if self.counters[0]&8==0 {return;}
                self.service_previous_frame=self.counters[0];self.background=ServiceMenu;
            }
            ServiceMenu=>self.service_menu_362a(),
            ServiceEasterEgg=>{
                if self.io_input[0]&0x80==0 {return;}
                self.service_leave_36c4();
            }
            ServiceLeaveWaitEight=>{
                if self.counters[0]<8 {return;}
                if self.io_input[0]&0x80==0 {
                    self.service_previous_frame=self.counters[0];self.background=ServiceMenu;
                } else {
                    self.c_spriteposn_regs_init_397c();self.service_crosshatch_36d9();
                    self.counters[0]=0;self.background=ServiceCrossWait;
                }
            }
            ServiceCrossWait=>{
                if self.counters[0]&0x80==0 {return;}
                self.background=ServiceCrossHold;
                if self.io_input[0]&0x80!=0 {self.service_leave_cross_3725();}
            }
            ServiceCrossHold=>{
                if self.io_input[0]&0x80!=0 {self.service_leave_cross_3725();}
            }
            ServiceLeaveWrap=>{
                if self.counters[0]!=0 {return;}
                self.background=ServiceConfigure;
            }
            ServiceConfigure=>{
                for parameter in self.io_parameters {self.input51.command(parameter);}
                self.background=ServiceConfigureWait;
            }
            ServiceConfigureWait=>{
                let input=self.input51.transfer_three();
                if input[0]>=0xa1 || input[0]&15>=10 {self.background=ServiceConfigure;return;}
                self.io_input=input;self.tile_ram[0x210]=0;
                self.j_game_init_02d3();
            }
            _=>unreachable!(),
        }
    }

    fn service_leave_36c4(&mut self) {
        self.counters[0]=0;self.background=Background::ServiceLeaveWaitEight;
    }
    fn service_leave_cross_3725(&mut self) {
        self.counters[0]=0xfe;self.background=Background::ServiceLeaveWrap;
    }

    /// 362A..36C3, retaining the seven-byte FIFO, both filter branches and
    /// the sixteen left-shift/carry calls in descending B order.
    fn service_menu_362a(&mut self) {
        let frame=self.counters[0];if frame==self.service_previous_frame {return;}
        self.service_previous_frame=frame;
        self.service_input.copy_within(0..7,1);
        if self.io_input[0]&0x80!=0 {self.service_leave_36c4();return;}
        self.service_input[0]=self.io_input[0];
        let buttons=!(self.service_input[0]|self.service_input[1])&self.service_input[2]&self.service_input[3];
        self.service_input[3]=buttons;
        self.service_input[4]=self.io_input[1];
        let stick=!(self.service_input[4]|self.service_input[5])&self.service_input[6]&self.service_input[7];
        self.service_input[7]=stick;
        let mut input=u16::from_be_bytes([buttons,stick]);
        for b in (1..=16).rev() {
            let carry=input&0x8000!=0;input=input.wrapping_shl(1);
            if carry {self.c_svc_test_input_hdlr_39e0(b);}
        }
        self.c_svc_updt_dsply();
        let timer=u16::from_le_bytes([self.shared_9260[18],self.shared_9260[19]]);
        if timer!=0 {
            let timer=timer.wrapping_sub(1);
            self.shared_9260[18..20].copy_from_slice(&timer.to_le_bytes());
            if timer==0 {self.c_svc_machine_ttls_erase_39c5();}
        }
        if self.service_input[0]&1!=0 {self.shared_9260[17]=0;return;}
        let stick=self.service_input[7]&15;if stick==0 {return;}
        let index=self.shared_9260[17] as usize;
        if SERVICE_EASTER_EGG_DATA[index]!=stick {self.shared_9260[17]=0;return;}
        self.shared_9260[17]=self.shared_9260[17].wrapping_add(1);
        if SERVICE_EASTER_EGG_DATA[index+1].wrapping_add(1)!=0 {return;}
        self.c_tileram_regs_clr_3962();self.c_spriteposn_regs_init_397c();
        let mut source=22;let mut destination=0x8042;
        for _ in 0..28 {self.c_svc_easteregg_3770(&mut source,&mut destination);}
        self.background=Background::ServiceEasterEgg;
    }

    fn c_tileram_regs_clr_3962(&mut self) {
        self.tile_ram.fill(0x24);self.colour_ram.fill(3);self.star_control[5]=7;
    }
    fn c_spriteposn_regs_init_397c(&mut self) {
        for object in &mut self.sprite_ram {object.flags=0xf1;object.write_odd_control(0xf1);}
    }

    /// 3987..39C4: original four calls, decimal complements and point counter.
    pub fn c_svc_machine_totals_3987(&mut self) {
        let mut totals=[0;12];totals[..2].copy_from_slice(&self.total_plays_bcd);
        totals[2..6].copy_from_slice(&self.total_score_bcd);
        totals[6..10].copy_from_slice(&self.game_time_bcd);
        totals[10..12].copy_from_slice(&self.bonus_count_bcd);
        let mut source=0;let mut destination=0x835e;let mut counter=2;
        self.c_39a1(&totals,&mut source,&mut destination,&mut counter,1);
        self.c_39a1(&totals,&mut source,&mut destination,&mut counter,3);
        self.c_39a1(&totals,&mut source,&mut destination,&mut counter,2);
        source+=1;
        self.c_39a1(&totals,&mut source,&mut destination,&mut counter,1);
    }
    fn c_39a1(&mut self,data:&[u8],source:&mut usize,destination:&mut u16,counter:&mut u8,count:u8) {
        self.c_39b4(data,source,destination,counter);
        for _ in 0..count {
            let mut value=0x99u8.wrapping_sub(data[*source]);let mut carry=0x99<data[*source];
            for _ in 0..4 {let next=value&1!=0;value=(value>>1)|(u8::from(carry)<<7);carry=next;}
            self.c_39b8(value,destination,counter);
            self.c_39b4(data,source,destination,counter);
        }
    }
    fn c_39b4(&mut self,data:&[u8],source:&mut usize,destination:&mut u16,counter:&mut u8) {
        let value=0x99u8.wrapping_sub(data[*source]);*source+=1;
        self.c_39b8(value,destination,counter);
    }
    fn c_39b8(&mut self,value:u8,destination:&mut u16,counter:&mut u8) {
        self.tile_ram[(*destination&0x3ff) as usize]=value&15;*destination=destination.wrapping_sub(32);
        *counter=counter.wrapping_sub(1);
        if *counter==0 {
            *counter=4;self.tile_ram[(*destination&0x3ff) as usize]=0x2a;
            *destination=destination.wrapping_sub(32);
        }
    }
    fn c_svc_machine_ttls_erase_39c5(&mut self) {
        let mut destination=0x835eu16;
        for _ in 0..23 {self.tile_ram[(destination&0x3ff) as usize]=0x24;destination=destination.wrapping_sub(32);}
    }

    /// 39E0..3A45. Service-credit, left/right selection and sound activation.
    pub fn c_svc_test_input_hdlr_39e0(&mut self,b:u8) {
        if b==15 {
            self.c_svc_machine_totals_3987();self.shared_9260[18..20].copy_from_slice(&900u16.to_le_bytes());return;
        }
        if b==2 {self.shared_9260[16]=self.shared_9260[16].wrapping_add(1);self.c_svc_test_sound_sel_39fc();return;}
        if b==4 {
            self.shared_9260[16]=if self.shared_9260[16]==0 {17} else {self.shared_9260[16]-1};
            self.c_svc_test_sound_sel_39fc();return;
        }
        if self.shared_9260[16]>=18 {self.shared_9260[16]=0;}
        self.combat.sound_requests.fill(0);self.audio.ram[0xa0..0xe0].fill(0);
        let destination=SERVICE_SOUND_DATA[8+self.shared_9260[16] as usize] as usize;
        self.audio.ram[destination]=1;self.combat.sound_requests[destination-0xa0]=1;
    }
    fn c_svc_test_sound_sel_39fc(&mut self) {
        let mut value=self.shared_9260[16];if value>=18 {value=0;}
        self.shared_9260[16]=value;let mut tens=0;
        if value>=10 {tens+=1;value-=10;}
        self.tile_ram[0x22e]=tens;self.tile_ram[0x20e]=value;self.c_text_out(0x3a51);
    }

    fn c_svc_easteregg_3770(&mut self,source:&mut usize,destination:&mut u16) {
        for _ in 0..3 {self.c_377e(source,destination);}
        *destination=destination.wrapping_add(5);
    }
    fn c_377e(&mut self,source:&mut usize,destination:&mut u16) {
        let mut value=SERVICE_EASTER_EGG_DATA[*source];
        for _ in 0..8 {
            let carry=value&0x80!=0;value=value.wrapping_shl(1);
            let tile=&mut self.tile_ram[(*destination&0x3ff) as usize];
            if carry {*tile=tile.wrapping_add(1);}
            *destination=destination.wrapping_add(1);
        }
        *source+=1;*destination=destination.wrapping_add(1);
    }
    fn service_crosshatch_36d9(&mut self) {
        let mut destination=0;
        for pair in [[0x28,0x27],[0x2d,0x2b],[0x28,0x2d],[0x27,0x2b]] {
            for _ in 0..16 {for tile in pair {self.tile_ram[destination]=tile;destination+=1;}}
        }
        let mut source=0x40;
        for _ in 0..0x340 {self.tile_ram[destination]=self.tile_ram[source];destination+=1;source+=1;}
        for source in 0..0x40 {self.tile_ram[destination]=self.tile_ram[source];destination+=1;}
    }
}
