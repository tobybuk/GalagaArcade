//! Source reconstruction of Galaga (Namco rev. B).
//! No CPU emulator, program counter, opcode decoder or instruction dispatcher.
//! Original task-table flow, gameplay, controller firmware and sound routines.
//! Unknown ROM-data addresses and invalid states remain explicit stops.
//! See PORTING.md for the exact scope and remaining entry-point adaptations.
pub mod formation;
pub mod motion;
pub mod rom_data;
pub mod video;
pub mod combat;
mod score;
mod tasks;
mod weapons;
pub mod attacks;
pub mod executive;
pub mod text;
pub mod capture;
pub mod stage;
pub mod highscore;
pub mod machine;
pub mod attract;
pub mod players;
pub mod sound;
pub mod starfield;
pub mod noise54;
pub mod noise_circuit;
pub mod configuration;
pub mod input51;
pub mod service;

use formation::Formation;
use motion::Motion;
use rom_data::*;

pub const FIGHTER: usize = 0x62 / 2;
pub const ROCKET_0: usize = 0x64 / 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum State {
    Rescued = 0,
    Standby = 1, HomeReturn = 2, Pattern = 3, Exploding = 4,
    Score = 5, Bomb = 6, Spawning = 7, Rogue = 8, Homing = 9,
    #[default] Inactive = 0x80,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Object {
    pub state: State,
    pub motion_slot: u8,
    pub code: u8,
    pub colour: u8,
    /// Original sprite coordinate registers, not Cartesian screen positions.
    pub x: u8,
    pub y: u16,
    pub flags: u8,
    /// Bits 1..7 of the odd control register. Bit 0 is the ninth Y bit in `y`.
    pub control_odd_upper:u8,
}

impl Object {
    pub fn odd_control(&self)->u8 {(self.control_odd_upper&0xfe)|((self.y>>8) as u8&1)}
    pub fn write_odd_control(&mut self,value:u8) {
        self.control_odd_upper=value&0xfe;
        self.y=(self.y&0xff)|(((value&1) as u16)<<8);
    }
}

pub struct Game {
    pub objects: [Object; 64],
    pub sprite_ram: [Object; 64],
    pub tile_ram: [u8; 1024],
    pub colour_ram: [u8; 1024],
    pub combat: combat::Combat,
    pub shared_9260: [u8; 32],
    pub no_restart_stage: u8,
    pub fighter_collision: u8,
    pub star_control: [u8; 6],
    pub main_task_enable: [u8; 32],
    pub sub_task_enable: [u8; 8],
    pub main_task_trace: [u16; 32],
    pub main_task_trace_len: usize,
    pub unused_dsw3: u8,
    pub bomb_rates: [[u8; 2]; 8],
    pub game_timers: [u8; 4],
    pub stage_parameters: [u8; 11],
    pub bomber_control: [u8; 10],
    pub continuous_bombing: u8,
    pub enemy_enabled: u8,
    pub flip_screen: u8,
    pub boss_sorties: [attacks::Sortie; 4],
    pub boss_escort_toggle: u8,
    pub capture_motion_slot: u8,
    pub bonus_bee_class: u8,
    pub bomb_reload: u8,
    pub stage_number: u8,
    pub machine_rank: u8,
    pub bomb_enable_flags: u8,
    pub nest_lr_inhibit: u8,
    pub bonus_bee_timer: u8,
    pub bonus_bee_colour: u8,
    /// Hardware refresh input for the original c_1000 lookup. Its whole-board
    /// phase remains an environment/timing validation boundary.
    pub refresh_counter: u8,
    pub stage_tokens: stage::StageTokens,
    pub machine_dsw6: u8,
    pub machine_dsw5:u8,
    pub dip_ports:[u8;8],
    pub io_parameters:[u8;8],
    pub challenge_hits_snapshot: u8,
    pub challenge_bonus_destination: u16,
    pub challenge_blink: u8,
    pub top5_ram: [u8; 80],
    pub top5_previous_frame: u8,
    pub credit_count: u8,
    pub reserve_task_enable: [u8;32],
    pub total_plays_bcd: [u8;2],
    pub total_score_bcd: [u8;4],
    pub game_time_bcd: [u8;4],
    pub coin_sound_count: u8,
    pub attract_state: u8,
    pub attract_text_index: u8,
    pub attract_setup_pending:bool,
    pub demo_timer: u8,
    pub demo_target: u8,
    pub demo_vector: u16,
    pub sprite_tiles_pointer: u16,
    pub attract_sound_dsw: u8,
    pub suspended_reserves: u8,
    pub suspended_player: [u8;64],
    pub suspended_objects: [u8;48],
    pub active_player_ram: [u8;64],
    pub formation_change_timer: u8,
    pub machine_cabinet: u8,
    pub machine_reserves: u8,
    pub audio: sound::Audio,
    pub formation_signage: u8,
    pub stars: starfield::Starfield,
    pub star_port_latch:u8,
    pub background: executive::Background,
    pub player_enemy_count: u8,
    pub attack_wave_enabled: u8,
    pub capture_connected: u8,
    pub capture_pause: u8,
    pub player_number: u8,
    pub two_player_game: bool,
    pub two_fighters: u8,
    pub game_state: u8,
    pub restart_stage: u8,
    pub reserve_fighters: u8,
    pub machine_bonus: [u8; 2],
    pub next_bonus: u8,
    pub bonus_count_bcd: [u8; 2],
    pub motions: [Motion; 12],
    pub formation: Formation,
    pub counters: [u8; 3],
    pub elapsed_frames: u32,
    pub wave_number: u8,
    pub entrance_complete: bool,
    pub fault: Option<u16>,
    pub shots: u16,
    pub rocket_attributes: [u8; 2],
    pub fighter_phase: u8,
    pub wave_sequence: [u8; 192],
    pub wave_cursor: usize,
    pub flying_count: u8,
    previous_active_motion_count: u8,
    object_count_accumulator: u8,
    pub active_enemy_count: u8,
    pub input51:input51::Input51,
    pub service_active:bool,
    pub service_input:[u8;8],
    pub service_previous_frame:u8,
    pub sub_diagnostic:[u8;24],
    pub io_input: [u8; 3],
    pixels: Vec<u8>,
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Self {
            objects: [Object::default(); 64], motions: [Motion::default(); 12],
            sprite_ram: [Object::default(); 64], tile_ram: [0x24; 1024], colour_ram: [0; 1024],
            combat: combat::Combat::default(),
            shared_9260: [0; 32], no_restart_stage: 0, fighter_collision: 0,
            star_control: [1, 0, 0, 0, 0, 6],
            main_task_enable: [0; 32], sub_task_enable: [0, 1, 1, 0, 1, 1, 0, 10],
            main_task_trace: [0; 32], main_task_trace_len: 0, unused_dsw3: 2,
            bomb_rates: [[0; 2]; 8],
            game_timers: [2, 0, 120, 0], stage_parameters: formation::stage_parameters(1, 3),
            bomber_control: [0x16, 2, 2, 0, 0, 0, 0, 0, 0, 0], continuous_bombing: 0,
            enemy_enabled: 3, flip_screen: 0,
            boss_sorties: [attacks::Sortie::default(); 4], boss_escort_toggle: 0,
            capture_motion_slot: 0, bonus_bee_class: 0, bomb_reload: STAGE_ONE_WAVES[0],
            stage_number: 1, machine_rank: 3, bomb_enable_flags: 0, nest_lr_inhibit: 0,
            bonus_bee_timer: 0, bonus_bee_colour: 0, refresh_counter: 0,
            stage_tokens: stage::StageTokens::default(), machine_dsw6: 0xff,
            machine_dsw5:3,dip_ports:[3,3,3,0,3,2,2,3],io_parameters:[1,1,1,1,1,2,3,0],
            challenge_hits_snapshot: 0, challenge_bonus_destination: 0, challenge_blink: 0,
            top5_ram: [0; 80], top5_previous_frame: 0, credit_count: 0,
            reserve_task_enable: [0;32], total_plays_bcd: [0;2], total_score_bcd: [0;4],
            game_time_bcd: [0;4], coin_sound_count: 0, attract_state: 0, attract_text_index: 0,
            attract_setup_pending:false,
            demo_timer: 0, demo_target: 0, demo_vector: 0, sprite_tiles_pointer: 0,
            attract_sound_dsw: 0, suspended_reserves: 0,
            suspended_player: [0;64], suspended_objects: [0;48], active_player_ram: [0;64],
            formation_change_timer: 0, machine_cabinet: 0, machine_reserves: 2,
            audio: sound::Audio::default(), formation_signage: 0,
            stars: starfield::Starfield::default(), star_port_latch: 6,
            background: executive::Background::GameRunner, player_enemy_count: 0, attack_wave_enabled: 1,
            capture_connected: 0, capture_pause: 0,
            player_number: 0, two_player_game: false, two_fighters: 0, game_state: 3,
            restart_stage: 0, reserve_fighters: 2,
            // Factory board configuration: 3 fighters, 20K/70K/every 70K.
            machine_bonus: [2, 7], next_bonus: 2, bonus_count_bcd: [0; 2],
            io_input: [0, 0x3f, 0x3f],
            formation: Formation::default(), counters: [0; 3], elapsed_frames: 0,
            wave_number: 0, entrance_complete: false, fault: None,
            shots: 0, rocket_attributes: [0; 2], fighter_phase: 0,
            wave_sequence: [0; 192], wave_cursor: 0, flying_count: 0,
            previous_active_motion_count: 0, object_count_accumulator: 0,
            active_enemy_count: 0, input51:input51::Input51::default(),
            service_active:false,service_input:[0;8],service_previous_frame:0,sub_diagnostic:[0;24],
            pixels: vec![0; video::WIDTH * video::HEIGHT * 4],
        };
        for &parameter in &[1,1,1,1,1,2,3,0] {game.input51.command(parameter);}
        // Prepared-entry verification fixture: the original start debit has
        // already disabled starts, and the physical inputs are idle.
        game.input51.ram[3]=2;
        for _ in 0..4 {game.input51.video_sample_0097([15;4]);}
        game.initialise_enemy_classes();
        game.gctl_stg_new_atk_wavs_init();
        game.initialise_score_tiles();
        game.initialise_top5();
        // Entry fixture at the stage-one game runner, after ship/intro setup.
        // Task slot identities, step values and dispatch order are original.
        for slot in [1, 5, 8, 10, 11, 12, 13, 15, 20, 21, 23] { game.main_task_enable[slot] = 1; }
        // main:0x134C (c_133A / fghtr_onscreen). Host enters this reconstructed slice
        // directly, after the credit/intro state machine (native fixture only).
        game.objects[FIGHTER] = Object { code: 6, colour: 9, x: 0x7a,
            y: 0x129, ..Object::default() };
        // main:0x127B (c_game_or_demo_init), two rockets followed by eight bombs.
        for n in 0..10 {
            game.objects[ROCKET_0 + n].code = 0x30;
            game.objects[ROCKET_0 + n].colour = if n < 2 { 9 } else { 11 };
            game.objects[ROCKET_0 + n].flags = if n < 2 { 0 } else { 1 };
        }
        game.render();
        game
    }
}

impl Game {
    /// main:0x23DD..0x259F. The original interleaves half the object set on
    /// alternate odd frames and publishes its enemy count every fourth frame.
    fn object_task(&mut self) {
        self.c_23e0(self.counters[0]);
    }

    pub(crate) fn c_23e0(&mut self,frame:u8) {
        if frame & 1 == 0 {
            if frame & 2 != 0 {
                self.active_enemy_count = self.object_count_accumulator;
                self.object_count_accumulator = 0;
            }
            return;
        }
        for i in (usize::from((frame & 2) / 2)..64).step_by(2) {
            let displayed = self.sprite_ram[i];
            let object = &mut self.objects[i];
            match object.state {
                State::Inactive => {}
                State::Rescued => {}
                State::Homing => {
                    let row = FORMATION_ORDINALS[i * 2] as usize / 2;
                    let column = FORMATION_ORDINALS[i * 2 + 1] as usize / 2;
                    let motion = &mut self.motions[object.motion_slot as usize];
                    motion.offset_x = self.formation.offsets[column];
                    motion.offset_y = self.formation.offsets[row];
                    self.object_count_accumulator = self.object_count_accumulator.wrapping_add(1);
                }
                State::HomeReturn | State::Standby => {
                    if object.state == State::HomeReturn {
                        if object.flags & 1 == 0 {
                            if object.code & 7 != 6 { object.code = object.code.wrapping_add(1); }
                            else { object.state = State::Standby; }
                        } else if object.code & 7 == 0 { object.flags &= !1; }
                        else { object.code = object.code.wrapping_sub(1); }
                    } else {
                        object.code = (object.code & !1) | u8::from(self.counters[2] & 2 != 0);
                        if self.enemy_enabled == 0 {
                            self.object_count_accumulator = self.object_count_accumulator.wrapping_add(1);
                            continue;
                        }
                    }
                    let row = FORMATION_ORDINALS[i * 2] as usize / 2;
                    let column = FORMATION_ORDINALS[i * 2 + 1] as usize / 2;
                    object.x = self.formation.coordinates[column] as u8;
                    let row=self.formation.coordinates[row];
                    object.y = row&0xff;
                    object.write_odd_control((row>>8) as u8);
                    self.object_count_accumulator = self.object_count_accumulator.wrapping_add(1);
                }
                State::Spawning => {
                    object.state = State::Pattern;
                    self.object_count_accumulator = self.object_count_accumulator.wrapping_add(1);
                }
                State::Pattern | State::Bomb => {
                    // main:254D explicitly sets bit 7 of the address to read
                    // the copied SFR registers, not the pending buffer bytes.
                    let y = (displayed.y >> 1) as u8;
                    if displayed.x < 0xf4 && (11..165).contains(&y) {
                        if object.state != State::Bomb {
                            self.object_count_accumulator = self.object_count_accumulator.wrapping_add(1);
                        }
                    } else {
                        if object.state == State::Pattern { self.motions[object.motion_slot as usize].flags = 0; }
                        object.state = State::Inactive;
                        object.x = 0;
                    }
                }
                State::Exploding => { self.case_24b2(i); }
                State::Score => { self.case_2535(i); }
                State::Rogue => { self.case_243c(i); }
            }
        }
    }

    /// One original video period: main-board tasks followed by motion-board
    /// tasks. Browser animation frequency is kept separate from this clock.
    /// Browser switches: left/right/fire/coin/start1/start2, bits 0..5.
    /// The original 51xx owns debounce, credits and both player input bytes.
    pub fn step(&mut self, buttons: u8) {
        if self.fault.is_some() { return; }
        let mut joystick=15;let mut switches=15;let mut coins=15;
        if buttons&1!=0 {joystick&=!8;}
        if buttons&2!=0 {joystick&=!2;}
        // One browser panel serves either player; both physical panel bytes
        // are still processed by the original independent controller paths.
        if buttons&4!=0 {switches&=!3;}
        if buttons&16!=0 {switches&=!4;}
        if buttons&32!=0 {switches&=!8;}
        if buttons&8!=0 {coins&=!1;}
        self.step_ports([joystick,joystick,switches,coins]);
    }

    /// Physical board inputs R0/R1 (sticks), R2 (fire/starts), R3 (coins/test).
    /// This permits independent cocktail controls and the original test switch.
    pub fn step_ports(&mut self,ports:[u8;4]) {
        if self.fault.is_some() {return;}
        self.input51.video_sample_0097(ports.map(|value|value&15));
        if !self.attract_setup_pending {self.background_run();}
        if self.fault.is_some() { return; }
        self.cpu0_rst38();
        if self.fault.is_some() { return; }
        // main:029C footer starts a three-byte 06xx input transfer. The main
        // tasks consume it on their next pass, and freeze skips this footer.
        if self.machine_dsw5&2!=0 && self.main_irq_enabled() && !self.attract_setup_pending {
            self.io_input=self.input51.transfer_three();
        }
        self.cpu1_rst38();
        if self.fault.is_some() { return; }
        if self.combat.sound_requests[0x19]!=0 {
            self.combat.sound_requests[0x19]=0;
            self.audio.noise.trigger_bang();
        }
        // Sound NMIs at raster lines 64 and 192, following the main/sub IRQ
        // at line 224: 104,128,32 lines = 312,384,96 samples at 48 kHz.
        self.audio.synthesise(0,312);
        self.sound_nmi();
        self.audio.synthesise(312,384);
        self.sound_nmi();
        self.audio.synthesise(696,96);
        self.stars.frame(self.star_port_latch);
        self.elapsed_frames = self.elapsed_frames.wrapping_add(1);
    }

    pub fn f_08d3(&mut self) {
        // sub:0x08D3: report last frame's active pool count, then recount.
        self.flying_count = self.previous_active_motion_count;
        self.previous_active_motion_count = 0;
        for slot in 0..12 {
            let mut motion = self.motions[slot];
            if !motion.active() { continue; }
            self.previous_active_motion_count += 1;
            let id = motion.object_id as usize / 2;
            let mut object = self.objects[id];
            if matches!(object.state, State::Pattern | State::Spawning | State::Homing) {
                if let Err(address) = self.mctl_fltpn_dspchr(&mut motion, &mut object, slot) {
                    self.fault = Some(address);
                }
            } else {
                self.case_0e49(&mut motion, &mut object);
            }
            self.motions[slot] = motion;
            self.objects[id] = object;
            if self.fault.is_some() { break; }
        }
    }

    pub fn render(&mut self) {
        video::render(&self.sprite_ram, &self.tile_ram, &self.colour_ram, &self.stars, self.flip_screen, &mut self.pixels);
    }

    pub fn pixels(&self) -> &[u8] { &self.pixels }
}

// Small browser ABI. JavaScript owns input/display/timing only. A game pointer
// is an opaque handle to a Rust allocation; it is never a ROM address.
#[cfg(target_arch = "wasm32")]
mod browser {
    use super::Game;
    #[no_mangle]
    pub extern "C" fn game_new() -> *mut Game {
        Box::into_raw(Box::new(Game::cold_boot_with_switches(0xf7,0x97)))
    }
    #[no_mangle]
    pub extern "C" fn game_new_with_switches(dswa:u32,dswb:u32) -> *mut Game {
        Box::into_raw(Box::new(Game::cold_boot_with_switches(dswa as u8,dswb as u8)))
    }
    #[no_mangle]
    pub unsafe extern "C" fn game_free(game: *mut Game) { drop(Box::from_raw(game)); }
    #[no_mangle]
    pub unsafe extern "C" fn game_step(game: *mut Game, buttons: u32) { (*game).step(buttons as u8); }
    #[no_mangle]
    pub unsafe extern "C" fn game_step_ports(game:*mut Game,panels:u32,auxiliary:u32) {
        (*game).step_ports([panels as u8&15,(panels>>4) as u8&15,
                           auxiliary as u8&15,(auxiliary>>4) as u8&15]);
    }
    #[no_mangle]
    pub unsafe extern "C" fn game_set_refresh_phase(game: *mut Game, sample: u32) {
        (*game).refresh_counter = sample as u8 & 0x7f;
    }
    #[no_mangle]
    pub unsafe extern "C" fn game_render(game: *mut Game) -> *const u8 {
        (*game).render();
        (*game).pixels().as_ptr()
    }
    #[no_mangle]
    pub unsafe extern "C" fn game_audio(game:*const Game)->*const f32 {(*game).audio.samples.as_ptr()}
    #[no_mangle]
    pub unsafe extern "C" fn game_status(game: *const Game, field: u32) -> u32 {
        let game = &*game;
        match field {
            0 => game.elapsed_frames,
            1 => game.wave_number as u32,
            2 => game.entrance_complete as u32,
            3 => game.active_enemy_count as u32,
            4 => game.fault.map_or(0, |a| a as u32 + 1),
            5 => game.shots as u32,
            6 => game.formation.pulsing as u32,
            7 => game.score_value(0),
            8 => game.combat.hits as u32,
            9 => game.stage_number as u32,
            10 => game.game_state as u32,
            11 => game.credit_count as u32,
            12 => game.attract_state as u32,
            16 => game.main_task_enable[21] as u32,
            17 => game.player_number as u32,
            18 => game.reserve_fighters as u32,
            19 => u16::from_be_bytes(game.total_plays_bcd) as u32,
            20 => u32::from(game.service_active),
            21 => game.two_fighters as u32,
            22 => game.combat.capture_boss_id as u32,
            23 => game.combat.capture_boss_flag as u32,
            24 => game.main_task_enable[24] as u32,
            25 => game.main_task_enable[28] as u32,
            26 => game.main_task_enable[17] as u32,
            27 => game.main_task_enable[29] as u32,
            28 => game.restart_stage as u32,
            29 => game.main_task_enable[20] as u32,
            30 => game.flying_count as u32,
            31 => game.active_enemy_count as u32,
            32 => u32::from(matches!(game.background,crate::executive::Background::Top5Entry)),
            // Read-only inspection for reproducible browser play checks.
            // It cannot alter object RAM, lives, stages, timing or game rules.
            0x100..=0x13f=>{
                let object=game.objects[(field-0x100) as usize];
                object.x as u32|((object.y as u32&0x1ff)<<8)|
                    ((object.state as u32)<<17)|((object.code as u32&0x7f)<<25)
            },
            0x200..=0x23f=>{
                let object=game.objects[(field-0x200) as usize];
                object.colour as u32|((object.flags as u32)<<8)|((object.odd_control() as u32)<<16)
            },
            0x300..=0x34f=>game.top5_ram[(field-0x300) as usize] as u32,
            0x400..=0x7ff=>game.tile_ram[(field-0x400) as usize] as u32,
            _ => 0,
        }
    }
}
