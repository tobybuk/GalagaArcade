use galaga_reconstructed::noise54::Noise54;

fn configured()->Noise54 {
    let mut noise=Noise54::default();
    for command in [0x30,0x40,0x00,0x02,0xdf,0x40,0x30,0x30,0x03,0xdf] {noise.command(command);}
    noise
}

#[test]
fn original_noise_configuration_stores_reversed_nibbles_and_waits_for_all_bytes() {
    let mut noise=Noise54::default();noise.command(0x30);
    assert!(!noise.iteration());assert_eq!(noise.iterations,0);
    for byte in [0x40,0,2,0xdf] {noise.command(byte);}
    assert!(noise.iteration());
    assert_eq!(&noise.ram[0x30..0x38],&[13,15,0,2,0,0,4,0]);
    for byte in [0x40,0x30,0x30,3,0xdf] {noise.command(byte);}
    assert_eq!(&noise.ram[0x38..0x40],&[13,15,0,3,3,0,3,0]);
    for byte in [0x60,0x12,0x34,0x56,0x78,0x9a] {noise.command(byte);}
    assert_eq!(&noise.ram[0x20..0x2a],&[9,10,7,8,5,6,3,4,1,2]);
    noise.command(0x70);assert_eq!(noise.ram[0x2f],0);
    noise.command(0x7f);assert_eq!(noise.ram[0x2f],15);
}

#[test]
fn noise_feedback_retains_original_five_nibble_shift_and_feedback() {
    let mut noise=Noise54::default();
    for expected in [[0,0,0,0,1],[0,0,0,8,9],[0,0,0,12,13],[0,0,0,14,15]] {
        noise.noise_a_0012();assert_eq!(&noise.ram[0x10..0x15],&expected);
    }
    for _ in 0..10000 {noise.iteration();}
    assert!(noise.ram.iter().all(|v|*v<16));assert!(noise.output.iter().all(|v|*v<16));
    for (source,previous,expected) in [(0,10,4),(2,10,5),(0,15,14),(2,15,15)] {
        noise.reset();noise.ram[0x10]=source;noise.ram[0x19]=previous;
        noise.noise_a_0012();assert_eq!(noise.ram[0x19],expected);
    }
}

#[test]
fn original_type_a_envelope_uses_borrow_chain_zero_phase_skip_and_ror_decay() {
    let mut noise=configured();noise.command(0x10);
    for _ in 0..64 {noise.envelope_a_0041();}
    assert_eq!((noise.ram[0x18],noise.ram[0x15]),(3,13));
    noise.envelope_a_0041();
    assert_eq!((noise.ram[0x18],noise.ram[0x15]),(1,15)); // zero-length silent phase skipped
    for volume in [11,8,6,4,3,2,1,0] {
        for _ in 0..528 {noise.envelope_a_0041();}
        assert_eq!(noise.ram[0x15],volume);
    }
    for _ in 0..528 {noise.envelope_a_0041();}
    assert_eq!(noise.ram[0x18],0);
}

#[test]
fn original_type_b_has_its_distinct_silent_phase_and_decay_period() {
    let mut noise=configured();noise.command(0x20);
    for _ in 0..48 {noise.envelope_b_00a7();}
    assert_eq!((noise.ram[0x1d],noise.ram[0x1a]),(3,13));
    noise.envelope_b_00a7();assert_eq!((noise.ram[0x1d],noise.ram[0x1a]),(2,0));
    for _ in 0..64 {noise.envelope_b_00a7();}
    assert_eq!((noise.ram[0x1d],noise.ram[0x1a]),(1,15));
    for _ in 0..784 {noise.envelope_b_00a7();}
    assert_eq!(noise.ram[0x1a],11);
}

#[test]
fn original_type_c_keeps_both_carry_additions_and_direct_volume_command() {
    let mut noise=Noise54::default();
    noise.ram[0x2b]=8;noise.ram[0x2f]=12;
    noise.ram[0x28]=15;noise.ram[0x29]=3;noise.ram[0x2a]=2;
    noise.ram[0x26]=15;noise.ram[0x27]=2;noise.ram[0x10]=1;
    noise.tone_c_00fc();assert_eq!(noise.output[2],0); // original TBIT/JMP gates the set half-cycle OFF
    assert_eq!((noise.ram[0x2a],noise.ram[0x2b]),(0,15));
    noise.ram[0x21..0x24].copy_from_slice(&[1,2,3]);noise.ram[0x20]=14;
    noise.command(0x50);
    assert_eq!(&noise.ram[0x2c..0x30],&[1,2,3,14]);assert_eq!(noise.ram[0x1f],2);
}

#[test]
fn firmware_cycle_metadata_matches_original_straight_line_and_gated_paths() {
    let mut noise=Noise54::default();
    let cycles=noise.noise_a_0012()+noise.envelope_a_0041()+noise.noise_b_0090()
        +noise.envelope_b_00a7()+noise.tone_c_00fc()+noise.envelope_c_0121();
    assert_eq!(cycles,90);
    noise.ram[0x1e]=1;noise.ram[0x19]=0;noise.ram[0x1a]=15;
    assert_eq!(noise.noise_b_0090(),19);assert_eq!(noise.output[1],0);
    noise.ram[0x1e]=0;
    assert_eq!(noise.noise_b_0090(),17);assert_eq!(noise.output[1],15);
    noise.ram[0x2b]=8;noise.ram[0x2f]=15;
    assert_eq!(noise.tone_c_00fc(),24);assert_eq!(noise.output[2],0);
}

#[test]
fn translated_noise_drives_original_filters_and_decay_to_silence() {
    let mut circuit=galaga_reconstructed::noise_circuit::NoiseCircuit::default();
    circuit.configure_after_self_test();
    for _ in 0..1920 {assert!(circuit.sample().abs()<1e-10);}
    circuit.trigger_bang();
    let mut peak=0.0f64;let mut last_peak=0.0f64;
    for i in 0..192000*6 {
        let value=circuit.sample();assert!(value.is_finite());peak=peak.max(value.abs());
        if i>192000*5 {last_peak=last_peak.max(value.abs());}
    }
    assert!(peak>0.01,"peak {peak}");assert!(last_peak<1e-6,"tail {last_peak}");
    assert_eq!((circuit.firmware.ram[0x18],circuit.firmware.ram[0x1d]),(0,0));
}
