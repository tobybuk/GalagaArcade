use galaga_reconstructed::{input51::Input51,Game};

fn configured(parameters:[u8;4])->Input51 {
    let mut io=Input51::default();io.command(1);
    for parameter in parameters {io.command(parameter);}
    io.command(2);io.command(3);io.command(0);
    for _ in 0..4 {io.video_sample_0097([15;4]);}
    io
}
fn pulse(io:&mut Input51,port:usize,mask:u8) {
    let mut input=[15;4];input[port]&=!mask;
    for _ in 0..2 {io.video_sample_0097(input);}
    for _ in 0..2 {io.video_sample_0097([15;4]);}
}
fn credits(io:&Input51)->u8 {io.ram[0x19]|io.ram[0x1a]<<4}

#[test]
fn command_parameter_direction_raw_switches_and_fourth_read_match_firmware() {
    let mut io=Input51::default();io.command(1);
    for value in [3,2,5,4] {io.command(value);}
    assert_eq!(&io.ram[0x10..0x14],&[4,5,2,3]);
    io.command(5);io.video_sample_0097([7,13,10,11]);
    assert_eq!(io.transfer_three(),[0xba,0xd7,0xff]);
    assert_eq!(io.ram[6],4);
    assert_eq!(io.read_024f(),0xba);assert_eq!(io.read_024f(),0xd7);
    assert_eq!(io.read_024f(),0xff);assert_eq!(io.read_024f(),0xff);
    assert_eq!(io.ram[6],4);
    io.command(4);assert_eq!(io.ram[2],0);
    for (raw,mapped) in [15,14,13,5,12,9,7,6,11,3,10,4,1,2,0,8].iter().enumerate() {
        assert_eq!(Input51::remap_0272(raw as u8),*mapped);
    }
}

#[test]
fn coin_and_button_filters_require_the_original_four_sample_pattern() {
    let mut io=configured([1;4]);
    io.video_sample_0097([15,15,15,14]);assert_eq!(credits(&io),0);
    io.video_sample_0097([15;4]);assert_eq!(credits(&io),0);
    for _ in 0..4 {io.video_sample_0097([15;4]);}
    io.video_sample_0097([15,15,15,14]);assert_eq!(credits(&io),0);
    io.video_sample_0097([15,15,15,14]);assert_eq!(credits(&io),1);
    for _ in 0..20 {io.video_sample_0097([15,15,15,14]);}
    assert_eq!(credits(&io),1);
    // All combinations of the four relevant samples for all four button bits.
    for packed in 0u32..65536 {
        let d=(packed&15) as u8;let a=((packed>>4)&15) as u8;
        let b=((packed>>8)&15) as u8;let c=((packed>>12)&15) as u8;
        io.ram[3]=2;io.ram[0x22]=a;io.ram[0x21]=b;io.ram[0x20]=c;
        io.video_sample_0097([15,15,d,15]);
        let mut expected=0;
        for bit in 0..4 {
            let edge=c&(1<<bit)!=0 && b&(1<<bit)!=0 && a&(1<<bit)==0 && d&(1<<bit)==0;
            if !edge {expected|=1<<bit;}
        }
        assert_eq!(io.ram[0x23],expected,"history {packed:04x}");
    }
}

#[test]
fn both_coin_slots_fractions_service_credit_decimal_carry_and_saturation() {
    let mut io=configured([3,2,2,3]);
    for _ in 0..2 {pulse(&mut io,3,1);assert_eq!(credits(&io),0);}
    pulse(&mut io,3,1);assert_eq!(credits(&io),2);
    pulse(&mut io,3,2);assert_eq!(credits(&io),2);
    pulse(&mut io,3,2);assert_eq!(credits(&io),5);
    pulse(&mut io,3,4);assert_eq!(credits(&io),6);
    io.ram[0x19]=9;io.ram[0x1a]=0;io.service_credit_01c6();
    assert_eq!(credits(&io),0x10);
    io.ram[0x19]=9;io.ram[0x1a]=9;io.service_credit_01c6();
    assert_eq!(credits(&io),0x99);
    io.ram[0x1b]=15;io.coin_a_0187();assert_eq!(io.ram[0x1b],15);
    // The original nibble DAA recomputes carry; preserve this protocol quirk.
    io.ram[0x13]=1;io.ram[0x1d]=0;io.ram[0x12]=7;
    io.ram[0x19]=9;io.ram[0x1a]=0;io.coin_a_0187();
    assert_eq!(credits(&io),6);
}

#[test]
fn start_priorities_disable_until_command_and_freeplay_debit_is_published_once() {
    let mut io=configured([1;4]);
    pulse(&mut io,3,1);pulse(&mut io,2,8);
    assert_eq!(credits(&io),1);assert_eq!(io.ram[3],0);
    pulse(&mut io,3,1);pulse(&mut io,2,12);
    assert_eq!(credits(&io),1);assert_eq!(io.ram[3],2); // start one has priority
    pulse(&mut io,2,4);assert_eq!(credits(&io),1);
    io.command(2);pulse(&mut io,2,4);assert_eq!(credits(&io),0);
    let mut free=configured([0;4]);assert_eq!(credits(&free),0xa0);
    free.video_sample_0097([15,15,7,15]);
    free.video_sample_0097([15,15,7,15]);
    assert_eq!(free.transfer_three()[0],0x98);
    assert_eq!(credits(&free),0xa0);assert_eq!(free.ram[3],2);
    free.video_sample_0097([15;4]);assert_eq!(free.transfer_three()[0],0xa0);
}

#[test]
fn original_fire_edge_and_held_bits_and_independent_cocktail_panel_are_retained() {
    let mut io=configured([1;4]);io.ram[3]=2;
    io.video_sample_0097([7,13,14,15]);
    assert_eq!(io.transfer_three(),[0,0x17,0x3d]);
    io.video_sample_0097([7,13,14,15]);
    assert_eq!(io.transfer_three(),[0,0x07,0x3d]);
    io.video_sample_0097([7,13,14,15]);
    assert_eq!(io.transfer_three(),[0,0x17,0x3d]);
    io.video_sample_0097([15,15,13,15]);
    io.video_sample_0097([15,15,13,15]);
    assert_eq!(io.transfer_three(),[0,0x3f,0x0f]);
    io.video_sample_0097([15,15,15,7]);
    assert_eq!(io.transfer_three()[0],0xbb);assert_eq!(io.ram[3],1);
}

#[test]
fn game_uses_controller_filter_and_previous_interrupt_input_transfer() {
    let mut g=Game::power_on();
    g.step(8);g.step(8);assert_eq!(g.credit_count,0);
    g.step(0);assert_eq!(g.credit_count,1);assert_eq!(g.game_state,2);
    g.step(0);g.step(0);
    g.step(16);g.step(16);assert_eq!(g.game_state,2);
    g.step(0);assert_eq!(g.game_state,3);assert_eq!(g.credit_count,0);
}
