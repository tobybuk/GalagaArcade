use galaga_reconstructed::sound::Audio;

#[test]
fn common_sound_state_frequency_and_wave_match_independently_compiled_c_each_nmi() {
    let reference=include_bytes!("../evidence/reference-sound.bin");
    let sounds=[1,2,3,4,8,9,10,11,12,13,15,16,17,19,20,21,22];
    let mut mismatches=Vec::new();
    for (scenario,sound) in sounds.into_iter().enumerate() {
        let mut audio=Audio::default();audio.ram[0xa0+sound]=1;
        for tick in 0..1024 {
            audio.c_nmi_proc(0);
            let registers:Vec<_>=audio.registers.iter().enumerate()
                .filter_map(|(i,value)|if [0x15,0x1a,0x1f].contains(&i) {None} else {Some(value)}).collect();
            let hash=audio.ram[..0x60].iter().chain(audio.ram[0xa0..0xe0].iter()).chain(registers)
                .fold(14695981039346656037u64,|hash,byte|(hash^*byte as u64).wrapping_mul(1099511628211));
            let index=(scenario*1024+tick)*8;
            let expected=u64::from_le_bytes(reference[index..index+8].try_into().unwrap());
            if hash!=expected {mismatches.push((sound,tick,hash,expected));break;}
        }
    }
    assert!(mismatches.is_empty(),"sound/NMI/hash differences: {mismatches:?}");
}
