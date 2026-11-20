//! Graphics data decoding and a software framebuffer for the browser.
//! Bit layouts, colour lookup and coordinate wiring follow the board layout
//! documented by MAME's BSD-3-Clause Galaga driver. No game logic lives here.
use crate::Object;

pub const WIDTH: usize = 224;
pub const HEIGHT: usize = 288;
const CHARACTERS: &[u8; 4096] = include_bytes!("../assets/characters.bin");
const SPRITES: &[u8; 8192] = include_bytes!("../assets/sprites.bin");
const PALETTE: &[u8; 32] = include_bytes!("../assets/palette.bin");
const CHARACTER_COLOURS: &[u8; 256] = include_bytes!("../assets/character-colours.bin");
const SPRITE_COLOURS: &[u8; 256] = include_bytes!("../assets/sprite-colours.bin");

fn bit(data: &[u8], offset: usize) -> u8 { (data[offset / 8] >> (7 - offset % 8)) & 1 }

pub fn sprite_pen(code: u8, x: usize, y: usize) -> u8 {
    let offset = (code as usize & 127) * 512 + (x / 4) * 64 + x % 4
        + (y % 8) * 8 + (y / 8) * 256;
    (bit(SPRITES, offset) << 1) | bit(SPRITES, offset + 4)
}

pub fn character_pen(code: u8, x: usize, y: usize) -> u8 {
    let offset = (code as usize & 127) * 128 + if x < 4 { 64 + x } else { x - 4 } + y * 8;
    (bit(CHARACTERS, offset) << 1) | bit(CHARACTERS, offset + 4)
}

/// Resistor-network colour conversion: 1k/470/220 ohm red/green,
/// 470/220 ohm blue. Lookup PROM index 15 is transparent for sprites.
pub fn rgb(index: u8) -> [u8; 3] {
    let p = PALETTE[index as usize & 31];
    let rgb3 = |v: u8| {
        let conductance = f64::from(v & 1) / 1000.0 + f64::from((v >> 1) & 1) / 470.0
            + f64::from((v >> 2) & 1) / 220.0;
        (255.0 * conductance / (1.0 / 1000.0 + 1.0 / 470.0 + 1.0 / 220.0) + 0.5) as u8
    };
    let blue = (255.0 * (f64::from((p >> 6) & 1) / 470.0 + f64::from((p >> 7) & 1) / 220.0)
        / (1.0 / 470.0 + 1.0 / 220.0) + 0.5) as u8;
    [rgb3(p), rgb3(p >> 3), blue]
}

fn plot(pixels: &mut [u8], x: i32, y: i32, colour: [u8; 3]) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 { return; }
    let offset = (y as usize * WIDTH + x as usize) * 4;
    pixels[offset..offset + 3].copy_from_slice(&colour);
}

fn sprite(pixels: &mut [u8], object: &Object,flip:u8) {
    let double_x = usize::from(object.flags & 4 != 0);
    let double_y = usize::from(object.flags & 8 != 0);
    let flip_x = usize::from(object.flags & 1 != 0)^usize::from(flip&1!=0);
    let flip_y = usize::from(object.flags & 2 != 0)^usize::from(flip&1!=0);
    let raw_x = object.y as i32 + 0x100*(object.odd_control()&2) as i32 - 40;
    let raw_y = ((257 - object.x as i32 - 16 * double_y as i32) & 255) - 32;
    // Zero is not a hardware disable bit: double-height sprites can wrap into
    // view at X=0. Clip the wired rectangle, including that original case.
    if raw_x>=288 || raw_x+16*(double_x as i32+1)<=0 ||
       raw_y>=224 || raw_y+16*(double_y as i32+1)<=0 {return;}
    for by in 0..=double_y {
        for bx in 0..=double_x {
            let code = object.code.wrapping_add(
                (((by ^ (double_y * flip_y)) * 2) + (bx ^ (double_x * flip_x))) as u8);
            for y in 0..16 {
                for x in 0..16 {
                    let pen = sprite_pen(code, if flip_x != 0 { 15 - x } else { x },
                                         if flip_y != 0 { 15 - y } else { y });
                    let colour = SPRITE_COLOURS[(object.colour as usize & 63) * 4 + pen as usize] & 15;
                    if colour == 15 { continue; }
                    // Rotate the board's 288x224 raster clockwise (ROT90).
                    plot(pixels, 223 - (raw_y + (by * 16 + y) as i32),
                         raw_x + (bx * 16 + x) as i32, rgb(colour));
                }
            }
        }
    }
}

/// Board tilemap mapper (36 by 28 raw tiles), followed by the physical
/// clockwise monitor rotation. Game code writes tile/colour RAM directly.
fn tilemap(pixels: &mut [u8], tiles: &[u8; 1024], colours: &[u8; 1024], flip:u8) {
    for row in 0..28usize {
        for column in 0..36usize {
            let adjusted_row = row + 2;
            let adjusted_column = column.wrapping_sub(2);
            let address = if adjusted_column & 0x20 != 0 {
                adjusted_row + ((adjusted_column & 31) << 5)
            } else { adjusted_column + (adjusted_row << 5) };
            let code = tiles[address] & 0x7f;
            let colour = colours[address] & 0x3f;
            for y in 0..8 {
                for x in 0..8 {
                    let pen = character_pen(code, x, y);
                    let indirect = (CHARACTER_COLOURS[colour as usize * 4 + pen as usize] & 15) | 16;
                    if indirect == 31 { continue; }
                    let (px,py)=(223-(row*8+y) as i32,(column*8+x) as i32);
                    plot(pixels,if flip&1!=0 {223-px} else {px},
                         if flip&1!=0 {287-py} else {py},rgb(indirect));
                }
            }
        }
    }
}

pub fn render(objects: &[Object; 64], tiles: &[u8; 1024], colours: &[u8; 1024], stars:&crate::starfield::Starfield, flip:u8, pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) { pixel.copy_from_slice(&[0, 0, 0, 255]); }
    for star in &stars.visible {
        // Two connected resistor bits. R/G's unconnected low bit is a pulldown.
        let rg=|v:u8| (255.0*((v&1) as f64/470.0+((v>>1)&1) as f64/220.0)
            /(1.0/1000.0+1.0/470.0+1.0/220.0)+0.5) as u8;
        let blue=(255.0*((star.colour>>4&1) as f64/470.0+(star.colour>>5&1) as f64/220.0)
            /(1.0/470.0+1.0/220.0)+0.5) as u8;
        plot(pixels,223-star.y as i32,star.x as i32,[rg(star.colour),rg(star.colour>>2),blue]);
    }
    for object in objects { sprite(pixels, object,flip); }
    tilemap(pixels, tiles, colours,flip);
}
