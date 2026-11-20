//! Headless trajectory export for reconstruction checks; no browser required.
use galaga_reconstructed::{Game, State};
fn main() {
    let mut game = Game::default();
    for frame in 0..4096 {
        if frame % 64 == 0 {
            println!("frame={frame},wave={},flying={},standby={},sliding={},pulse={},fault={:?}",
                game.wave_number, game.flying_count,
                game.objects.iter().filter(|o| o.state == State::Standby).count(),
                game.formation.sliding, game.formation.pulsing, game.fault);
        }
        game.step(0);
        if let Some(address) = game.fault {
            println!("stopped at original routine {address:04x}, after {} complete frames", game.elapsed_frames);
            break;
        }
    }
    if let Some(path) = std::env::args().nth(1) {
        game.render();
        let mut data = format!("P6\n{} {}\n255\n", 224, 288).into_bytes();
        for rgba in game.pixels().chunks_exact(4) { data.extend_from_slice(&rgba[..3]); }
        std::fs::write(path, data).unwrap();
    }
}
