use galaga_reconstructed::{Game, FIGHTER};

#[test]
fn common_sorties_match_unmodified_c_reference_each_frame() {
    let reference = include_bytes!("../evidence/reference-flight.bin");
    for (scenario, (id, path, colour)) in [(8,0x34f,3), (0x40,0x3a9,2),
                                         (0x30,0x411,0), (0x32,0x411,0)].into_iter().enumerate() {
        let mut game = Game::default();
        game.main_task_enable.fill(0);
        game.objects[FIGHTER].x = 122;
        game.objects[id / 2].x = 100; game.objects[id / 2].y = 100;
        game.objects[id / 2].code = 6; game.objects[id / 2].colour = colour;
        game.c_1079(id as u8 | if scenario == 3 { 0x80 } else { 0 }, path);
        for frame in 0..1024 {
            game.counters[0] = frame as u8;
            game.f_08d3();
            assert_eq!(game.fault, None);
            let o = game.objects[id / 2];
            let m = game.motions[0];
            let mut data = vec![o.state as u8, o.code, o.colour, o.x, o.y as u8, (o.y >> 8) as u8 & 1, o.flags];
            for word in [m.y, m.x, m.angle, m.path] { data.extend_from_slice(&word.to_le_bytes()); }
            data.extend_from_slice(&[m.duration,m.rotation as u8,m.flags & 1,m.home_y,m.home_x,
                                     m.odd_displacement,m.even_displacement,m.bomb_counter,m.bomb_flags]);
            let hash = data.iter().fold(14_695_981_039_346_656_037u64,
                |hash, &b| (hash ^ b as u64).wrapping_mul(1_099_511_628_211));
            let offset = (scenario * 1024 + frame) * 8;
            let expected = u64::from_le_bytes(reference[offset..offset + 8].try_into().unwrap());
            assert_eq!(hash, expected, "sortie {scenario}, frame {frame}, motion {m:?}, object {o:?}");
        }
    }
}
