use galaga_reconstructed::{combat::Combat, Game, Object, State, ROCKET_0};

#[test]
fn common_rocket_paths_match_unmodified_reference_for_every_target_coordinate() {
    let reference = include_bytes!("../evidence/reference-combat.bin");
    let mut game = Game::default();
    for (scenario, (colour, state)) in [(2, State::Standby), (3, State::Standby),
                                       (2, State::Pattern), (3, State::Pattern),
                                       (0, State::Standby)].into_iter().enumerate() {
        let mut hash = 14_695_981_039_346_656_037u64;
        for x in 0..=255 {
            for y in 0..512 {
                game.combat = Combat::default();
                game.objects[4] = Object { colour, state, x, y, motion_slot: 0, code: 0x18, ..Object::default() };
                game.motions[0].flags = 1;
                game.objects[ROCKET_0] = Object { x: 128, y: 256, ..Object::default() };
                game.hitd_det_rckt(ROCKET_0);
                let mut add = |b: u8| { hash = (hash ^ b as u64).wrapping_mul(1_099_511_628_211); };
                for b in game.combat.hits.to_le_bytes() { add(b); }
                for b in [game.objects[ROCKET_0].x, game.objects[ROCKET_0].y as u8,
                          (game.objects[ROCKET_0].y >> 8) as u8 & 1,
                          game.objects[4].colour, game.combat.hit_notifications[4], game.motions[0].flags] { add(b); }
                for b in game.combat.hit_multipliers { add(b); }
                for b in &game.combat.sound_requests[..8] { add(*b); }
                add(game.combat.flying_hits);
                add(game.combat.flight_hits_remaining);
            }
        }
        let expected = u64::from_le_bytes(reference[scenario * 8..scenario * 8 + 8].try_into().unwrap());
        assert_eq!(hash, expected, "common-hit scenario {scenario}");
    }
}
