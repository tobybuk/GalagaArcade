use galaga_reconstructed::{sound::Audio,starfield::Starfield,Game};

#[test]
fn all_original_sound_requests_run_and_single_shot_themes_finish() {
    for index in 0..23 {
        let mut sound=Audio::default();sound.ram[0xa0+index]=1;
        let mut heard=false;
        for tick in 0..6000 {
            sound.c_nmi_proc(if tick/128&1==0 {1} else {255});
            heard|=sound.registers[0x15]|sound.registers[0x1a]|sound.registers[0x1f]!=0;
        }
        assert!(heard,"request {index:02x}");
        if ![0,5,6,9,0x10,0x11].contains(&index) {assert_eq!(sound.ram[0xa0+index],0,"request {index:02x}");}
    }
}

#[test]
fn original_coin_count_repeats_and_top_score_sound_transitions_to_exit_tune() {
    let mut sound=Audio::default();sound.ram[0x79]=3;
    let mut counts=Vec::new();
    for _ in 0..4000 {
        sound.c_nmi_proc(1);
        if counts.last()!=Some(&sound.ram[0xa8]) {counts.push(sound.ram[0xa8]);}
    }
    assert_eq!(counts,[3,2,1,0]);assert_eq!(sound.ram[0x79],0);
    sound.ram[0xac]=1;
    let mut exit_seen=false;
    for _ in 0..6000 {sound.c_nmi_proc(1);exit_seen|=sound.ram[0xb6]!=0;}
    assert!(exit_seen);assert_eq!((sound.ram[0xac],sound.ram[0xb6]),(0,0));
}

#[test]
fn rev_b_ship_hit_skips_lower_priority_themes_and_uses_original_clear_ranges() {
    let mut sound=Audio::default();
    sound.ram[0xa7]=1;sound.ram[0xab]=1;sound.ram[0xb7]=0;
    sound.c_nmi_proc(1);
    assert_eq!(sound.ram[0xd3],0); // opening-theme active flag not entered while sound 7 runs
    sound.ram[0xa9]=1;sound.ram[0xaa]=0x44;
    sound.ram[0xab..0xb7].fill(0x22);
    sound.ram[0xc8]=0x13;sound.ram[0xca]=0x14;sound.ram[0xd7]=0x15;
    sound.ram[0x74]=7;sound.ram[0x78]=1;sound.c_04a2();
    assert_eq!((sound.ram[0xa8],sound.ram[0xa9],sound.ram[0xaa]),(0,0,0x44));
    assert!(sound.ram[0xab..0xb7].iter().all(|v|*v==0));
    assert_eq!((sound.ram[0xc8],sound.ram[0xca],sound.ram[0xd7]),(0x13,0x14,0x15));
}

#[test]
fn original_voice_envelopes_masks_and_browser_wave_output_are_preserved() {
    let mut sound=Audio::default();sound.ram[0x74]=15;sound.ram[0x75]=31;sound.ram[0x77]=2;
    sound.c_0550();
    assert_ne!(sound.ram[0x6f],0);
    sound.ram[0xa0+15]=1;sound.c_nmi_proc(1);sound.synthesise(0,792);
    assert!(sound.samples.iter().all(|v|v.is_finite()&&v.abs()<=1.0));
    assert!(sound.samples.iter().any(|v|v.abs()>0.0));
    assert!(sound.registers.iter().all(|v|*v<16));
}

#[test]
fn supplied_envelope_branches_use_counter_complement_and_subtract_ten_before_negation() {
    // Supplied sound:061E..065E. The C aid uses a stale A for CPL and omits
    // SUB 0A before NEG; its volume output is deliberately excluded as oracle.
    for (sound,index,kind,threshold) in [(1,1,0u8,0u8),(10,7,1,6),(9,13,2,4)] {
        let mut audio=Audio::default();audio.ram[0x74]=sound;audio.ram[0x75]=index;audio.ram[0x77]=0;
        for count in 0..=255u8 {
            audio.ram[index as usize]=count.wrapping_sub(1);audio.ram[0x30+index as usize]=0;
            audio.c_0550();
            let expected=if kind!=0 && count<6 {
                if kind==1 {count.wrapping_mul(2)} else {!count}
            } else if threshold==0 || count<threshold {10}
            else {let delta=count.wrapping_sub(threshold);if delta>=10 {0} else {10-delta}};
            assert_eq!(audio.ram[0x65],expected,"envelope {kind}, count {count}");
        }
    }
}

#[test]
fn hardware_star_clock_is_stationary_at_seven_and_does_not_advance_on_redraw() {
    let mut stars=Starfield::default();stars.frame(7);
    assert_eq!(stars.lfsr,0x7fff);let visible=stars.visible.clone();
    stars.frame(7);assert_eq!(stars.visible,visible);
    stars.frame(6);assert_ne!(stars.lfsr,0x7fff);assert_ne!(stars.visible,visible);
    let mut g=Game::default();g.stars=stars;let lfsr=g.stars.lfsr;
    g.render();g.render();assert_eq!(g.stars.lfsr,lfsr);
}
