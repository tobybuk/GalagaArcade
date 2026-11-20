//! Original background executive. Busy-waits yield to the browser's next board
//! video period; named continuations retain each original return/call boundary.
//! These are Rust game states, not an instruction/register execution engine.
use crate::{Game, State, FIGHTER};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageAfter { Ready, Respawn, PlayerUp }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    AttractWait,
    ReadyWait,
    OpeningThemeWait,
    HaltFirstInterrupt,
    HaltIoComplete,
    PlayerChangeFlyingWait,
    PlayerChangeRetreatWait,
    PlayerChangeTokens,
    PlayerChangeReturnWait,
    GameRunner,
    RestartWait,
    RestartCapturedWait,
    RespawnWaitForBugs,
    RespawnDelay,
    GameOverDelayOne,
    GameOverDelayTwo,
    GameOverBeamWait,
    GameOverResultsWait,
    StageTokens(StageAfter),
    StageSplashWait(StageAfter),
    ChallengeWaitOne,
    ChallengeWaitTwo,
    ChallengeWaitThree,
    ChallengeBonusWait,
    ChallengeBlinkReady,
    ChallengeBlinkLeave,
    ChallengeAwardWaitOne,
    ChallengeAwardWaitTwo,
    Top5IntroWait,
    Top5Entry,
    Top5FinishWait,
    GameOverSoundWait,
    ServiceWaitTwo,
    ServiceWaitEight,
    ServiceMenu,
    ServiceEasterEgg,
    ServiceLeaveWaitEight,
    ServiceCrossWait,
    ServiceCrossHold,
    ServiceLeaveWrap,
    ServiceConfigure,
    ServiceConfigureWait,
}

impl Game {
    pub fn background_run(&mut self) {
        // Continue straight-line calls in this video period until an original
        // wait needs an interrupt or the game-runner reaches its next pass.
        loop {
            match self.background {
                Background::ServiceWaitTwo|Background::ServiceWaitEight|
                Background::ServiceMenu|Background::ServiceEasterEgg|
                Background::ServiceLeaveWaitEight|Background::ServiceCrossWait|
                Background::ServiceCrossHold|Background::ServiceLeaveWrap|
                Background::ServiceConfigure|Background::ServiceConfigureWait=>{self.service_continue();return;},
                Background::AttractWait => {
                    if self.game_state == 1 { return; }
                    self.c_1242_init_taskman_structs();
                    self.c_sctrl_playfld_clr();
                    self.motions.fill(Default::default());
                    self.c_sctrl_sprite_ram_clr();
                    self.l_game_state_ready();
                }
                Background::ReadyWait => {
                    if self.game_state == 2 { return; }
                    self.game_start();
                    return;
                }
                Background::OpeningThemeWait => {
                    if self.game_timers[3] != 0 { return; }
                    self.combat.hit_multipliers.fill(0);
                    self.suspended_objects.fill(0);
                    self.c_string_out(11,Some(0x83b0));
                    self.suspended_player[0x20]=1;
                    self.next_bonus=self.machine_bonus[0];
                    self.suspended_player[0x1e]=self.machine_bonus[0];
                    self.stg_init_splash(StageAfter::PlayerUp);
                }
                Background::HaltFirstInterrupt => {
                    // The 06xx data-transfer HALT is an environment boundary.
                    // Three original 51xx mode bytes (2,2,2) select credit mode.
                    for _ in 0..3 {self.input51.command(2);}
                    self.background=Background::HaltIoComplete;
                    return;
                }
                Background::HaltIoComplete => { self.g_halt_complete(); }
                Background::PlayerChangeFlyingWait => {
                    if self.active_enemy_count!=0 && self.flying_count!=0 {return;}
                    self.formation_change_timer=0; self.main_task_enable[14]=1;
                    self.background=Background::PlayerChangeRetreatWait;
                    return;
                }
                Background::PlayerChangeRetreatWait => {
                    if self.main_task_enable[14]!=0 {return;}
                    self.player_change_exchange();
                }
                Background::PlayerChangeTokens => {
                    if !self.stage_tokens_continue() {return;}
                    if self.player_enemy_count==0 {self.stg_init_splash(StageAfter::PlayerUp);}
                    else {
                        self.c_string_out(3,None); self.formation_change_timer=0x80;
                        self.main_task_enable[14]=1;
                        self.background=Background::PlayerChangeReturnWait;
                        return;
                    }
                }
                Background::PlayerChangeReturnWait => {
                    if self.main_task_enable[14]!=0 {return;}
                    self.plyr_respawn_plyrup();
                }
                Background::GameRunner => {
                    self.gctl_supv_score();
                    if self.main_task_enable[8] | self.active_enemy_count == 0 {
                        self.combat.sound_requests[0] = 0;
                        self.gctl_stg_restart_hdlr();
                    } else if self.restart_stage != 0 {
                        self.attack_wave_enabled = 0;
                        self.gctl_stg_restart_hdlr();
                    } else { return; }
                }
                Background::RestartWait => {
                    if self.main_task_enable[0x1d] != 0 {
                        self.restart_stage = 0;
                        self.sub_task_enable[5] = 1;
                        if self.active_enemy_count != 0 { self.background = Background::GameRunner; return; }
                        self.background = Background::RestartCapturedWait;
                        continue;
                    }
                    if self.game_timers[3] != 0 { return; }
                    self.gctl_supv_score();
                    self.player_enemy_count = self.active_enemy_count;
                    if self.restart_stage | self.active_enemy_count != 0 {
                        self.gctl_plyr_terminate();
                    } else {
                        if self.is_challenge_stage() { self.gctl_chllng_stg_end(); }
                        else { self.stg_init_splash(StageAfter::Ready); }
                    }
                }
                Background::RestartCapturedWait => {
                    if self.main_task_enable[0x1d] != 0 { return; }
                    self.stg_init_splash(StageAfter::Ready);
                }
                Background::RespawnWaitForBugs => {
                    if self.flying_count != 0 { return; }
                    self.c_133a_fghtr_onscreen();
                    // gctl_plyr_respawn_wait: add, compare and saturate AFTER
                    // wrapping the byte, exactly as the original does.
                    let mut timer = self.game_timers[2].wrapping_add(0x1e);
                    if timer >= 0x78 { timer = 0x78; }
                    self.game_timers[2] = timer;
                    self.c_tdelay_3();
                    self.background = Background::RespawnDelay;
                }
                Background::RespawnDelay => {
                    if self.game_timers[3] != 0 { return; }
                    self.plyr_respawn_rdy();
                }
                Background::GameOverDelayOne => {
                    if self.game_timers[3] != 0 { return; }
                    self.c_tdelay_3();
                    self.background = Background::GameOverDelayTwo;
                }
                Background::GameOverDelayTwo => {
                    if self.game_timers[3] != 0 { return; }
                    self.background = Background::GameOverBeamWait;
                }
                Background::GameOverBeamWait => {
                    if self.main_task_enable[0x18] != 0 { return; }
                    self.motions.fill(Default::default());
                    self.c_sctrl_sprite_ram_clr();
                    self.c_sctrl_playfld_clr();
                    self.c_string_out(0x15, None);
                    self.c_string_out(0x16, None);
                    self.c_text_out_i_to_d(self.shots, 0x8132);
                    self.c_string_out(0x18, None);
                    self.c_text_out_i_to_d(self.combat.hits, 0x8135);
                    self.c_string_out(0x19, None);
                    let destination = self.c_0a72_puts_hitmiss_ratio();
                    self.c_string_out(0x1a, Some(destination));
                    self.game_timers[2] = 14;
                    self.background = Background::GameOverResultsWait;
                }
                Background::GameOverResultsWait => {
                    if self.game_timers[2] != 0 { return; }
                    self.c_sctrl_playfld_clr();
                    self.c_top5_dlg_proc();
                }
                Background::StageTokens(after) => {
                    if !self.stage_tokens_continue() { return; }
                    self.background = Background::StageSplashWait(after);
                }
                Background::StageSplashWait(after) => {
                    if self.game_timers[2] != 0 { return; }
                    self.stg_init_env();
                    if self.fault.is_some() { return; }
                    if self.machine_dsw6 & 2 == 0 {
                        self.c_string_out(11, Some(0x83b0));
                        self.stg_init_splash(after);
                        continue;
                    }
                    match after {
                        StageAfter::Ready => self.plyr_respawn_rdy(),
                        StageAfter::Respawn => self.c_player_respawn(),
                        StageAfter::PlayerUp => {
                            self.c_string_out(self.player_number.wrapping_add(4), Some(0x826e));
                            self.c_player_respawn();
                        }
                    }
                }
                Background::ChallengeWaitOne => {
                    if self.game_timers[3] != 0 { return; }
                    self.c_string_out(8, None);
                    self.c_tdelay_3(); self.background = Background::ChallengeWaitTwo;
                }
                Background::ChallengeWaitTwo => {
                    if self.game_timers[3] != 0 { return; }
                    self.c_text_out_i_to_d(self.challenge_hits_snapshot as u16, 0x8110);
                    self.c_tdelay_3(); self.background = Background::ChallengeWaitThree;
                }
                Background::ChallengeWaitThree => {
                    if self.game_timers[3] != 0 { return; }
                    if self.combat.flying_hits == 40 {
                        self.challenge_blink = 7; self.background = Background::ChallengeBlinkReady;
                    } else {
                        self.challenge_bonus_destination = self.c_string_out(9, None);
                        self.c_tdelay_3(); self.background = Background::ChallengeBonusWait;
                    }
                }
                Background::ChallengeBonusWait => {
                    if self.game_timers[3] != 0 { return; }
                    let mut destination = self.challenge_bonus_destination;
                    let hits = self.combat.flying_hits;
                    if hits != 0 {
                        destination = self.c_text_out_i_to_d(hits as u16, destination);
                        self.tile_ram[(destination & 0x3ff) as usize] = 0;
                        destination = destination.wrapping_sub(32);
                    }
                    self.tile_ram[(destination & 0x3ff) as usize] = 0;
                    self.challenge_award(hits);
                }
                Background::ChallengeBlinkReady => {
                    if self.counters[0] & 15 != 0 { return; }
                    self.c_string_out(if self.challenge_blink & 1 == 0 { 11 } else { 12 }, None);
                    self.background = Background::ChallengeBlinkLeave;
                }
                Background::ChallengeBlinkLeave => {
                    if self.counters[0] & 15 == 0 { return; }
                    self.challenge_blink = self.challenge_blink.wrapping_sub(1);
                    if self.challenge_blink == 0 {
                        self.c_string_out(13, None);
                        self.challenge_award(100);
                    } else { self.background = Background::ChallengeBlinkReady; }
                }
                Background::ChallengeAwardWaitOne => {
                    if self.game_timers[3] != 0 { return; }
                    self.c_tdelay_3(); self.background = Background::ChallengeAwardWaitTwo;
                }
                Background::ChallengeAwardWaitTwo => {
                    if self.game_timers[3] != 0 { return; }
                    self.c_string_out(11, Some(0x83b0));
                    self.c_string_out(11, Some(0x83b3));
                    self.c_string_out(11, None);
                    self.stg_init_splash(StageAfter::Ready);
                }
                Background::Top5IntroWait => {
                    if self.game_timers[2] != 0 { return; }
                    self.game_timers[2] = 40;
                    self.l_30b5_next_char_selectn();
                    return;
                }
                Background::Top5Entry => { self.top5_entry_frame(); return; }
                Background::Top5FinishWait => {
                    if self.counters[0] != 0 { return; }
                    self.top5_return();
                }
                Background::GameOverSoundWait => {
                    if self.combat.sound_requests[0x0c] | self.combat.sound_requests[0x16] != 0 {
                        if self.combat.sound_requests[0x0c] != 0 { self.combat.sound_requests[0x0c] = 1; }
                        return;
                    }
                    self.c_sctrl_playfld_clr();
                    if self.two_player_game && self.suspended_reserves != 0xff {
                        self.background=Background::PlayerChangeFlyingWait;
                    } else { self.g_halt(); }
                    return;
                }
            }
            if self.fault.is_some() { return; }
        }
    }

    /// main:049E. The stack-pop removes the stage supervisor's return; the
    /// named continuation preserves that jump into the background executive.
    fn gctl_stg_restart_hdlr(&mut self) {
        self.game_timers[3] = 4;
        self.background = Background::RestartWait;
    }

    /// main:04E2, decrement BEFORE testing the old count for game over.
    fn gctl_plyr_terminate(&mut self) {
        let reserves = self.reserve_fighters;
        self.reserve_fighters = reserves.wrapping_sub(1);
        if reserves == 0 {
            if self.two_player_game {
                self.c_string_out(self.player_number.wrapping_add(4), Some(0x824e));
            }
            self.c_string_out(2, None);
            self.c_tdelay_3();
            self.background = Background::GameOverDelayOne;
            return;
        }
        if self.two_player_game {
            if self.suspended_reserves==0xff || self.restart_stage!=1 {self.plyr_respawn_plyrup();}
            else {self.background=Background::PlayerChangeFlyingWait;}
            return;
        }
        // plyr_respawn_1up calls the new-stage splash if the collision removed
        // the last enemy, otherwise it enters the existing-stage respawn.
        if self.player_enemy_count == 0 { self.stg_init_splash(StageAfter::Respawn); return; }
        self.c_player_respawn();
    }

    /// main:1331 (c_tdelay_3), the original divided-clock timer, not host time.
    fn c_tdelay_3(&mut self) { self.game_timers[3] = 3; }

    /// main:064A, preserve the initial DE hit snapshot through the first waits.
    fn gctl_chllng_stg_end(&mut self) {
        self.challenge_hits_snapshot = self.combat.flying_hits;
        self.combat.sound_requests[if self.combat.flying_hits == 40 { 0x14 } else { 0x0e }] = 1;
        self.c_tdelay_3(); self.background = Background::ChallengeWaitOne;
    }

    /// main:06BA, add the original 100-point multiplier, score, then two waits.
    fn challenge_award(&mut self, multiplier: u8) {
        self.combat.hit_multipliers[15] = self.combat.hit_multipliers[15].wrapping_add(multiplier);
        self.gctl_supv_score();
        self.c_tdelay_3(); self.background = Background::ChallengeAwardWaitOne;
    }

    /// main:133D, c_player_respawn followed by its c_133A wait entry.
    fn c_player_respawn(&mut self) {
        self.main_task_enable[0x14] = 1;
        if self.tile_ram[0x270] == 0x24 { self.c_string_out(3, None); }
        self.background = Background::RespawnWaitForBugs;
    }

    fn plyr_respawn_plyrup(&mut self) {
        self.c_string_out(self.player_number.wrapping_add(4),Some(0x826e));
        self.c_player_respawn();
    }

    /// main:134C..137D, exact stores after the flying-count busy-wait.
    pub(crate) fn c_133a_fghtr_onscreen(&mut self) {
        self.draw_resv_ships();
        let fighter = &mut self.objects[FIGHTER];
        fighter.code = 6; fighter.colour = 9;
        fighter.x = 0x7a;
        fighter.y = if self.flip_screen & 1 == 0 { 0x129 } else { 0x37 };
        fighter.write_odd_control((fighter.y>>8) as u8);
        fighter.flags = 0;
        self.restart_stage = 0;
        self.star_control[0] = 1;
    }

    /// main:0632, shared tail of the original new-stage and respawn paths.
    fn plyr_respawn_rdy(&mut self) {
        self.main_task_enable[0x15] = 1;
        self.sub_task_enable[5] = 1;
        self.attack_wave_enabled = 1;
        self.c_string_out(0x0b, Some(0x83b0));
        self.c_string_out(0x0b, Some(0x83ae));
        self.background = Background::GameRunner;
    }

    /// main:003C, c_sctrl_sprite_ram_clr. Code/colour RAM is NOT cleared by
    /// this routine. Both bytes of each disposition entry become 0x80.
    pub fn c_sctrl_sprite_ram_clr(&mut self) {
        for object in &mut self.objects {
            object.x = 0; object.y = 0; object.flags = 0;
            object.write_odd_control(0);
            object.state = State::Inactive; object.motion_slot = 0x80;
        }
    }
}
