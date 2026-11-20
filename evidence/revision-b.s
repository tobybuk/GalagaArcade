; Supplied Namco rev. B: corrected routines, assembled for byte verification.
.module revision_b
.area ROM (ABS,OVR)
.org 0x137E
draw_resv_ships:
    ld a,(0x9820)
    cpl
    add a,#9
    ld e,a
    ld d,#0x49
    ld hl,#0x801D
    call draw_resv_ship_tile
    dec l
    call draw_resv_ship_tile
    set 5,l
    inc l
    call draw_resv_ship_tile
    dec l
draw_resv_ship_tile:
    push hl
    inc d
    ld c,d
    ld b,#8
reserve_loop:
    ld a,b
    cp e
    jr nz,reserve_test_existing
    ld c,#0x24
reserve_test_existing:
    ld a,(hl)
    cp #0x36
    jr c,reserve_write
    cp #0x4A
    jr c,reserve_advance
reserve_write:
    ld (hl),c
reserve_advance:
    dec l
    dec l
    djnz reserve_loop
    pop hl
    ret
.org 0x1EAA
f_1eaa:
    ld a,(0x92A0)
    and #1
    add a,#2
    ld b,a
    ld a,(0x9215)
    and a
    ld a,b
    jr z,bomb_dy_ready
    neg
bomb_dy_ready:
    ld ixh,a
    ld l,#0x68
    ld de,#0x92B0
    ld ixl,#8
bomb_loop:
    ld h,#0x8B
    ld a,(hl)
    cp #0x30
    jr nz,bomb_inactive
    ld h,#0x93
    ld a,(hl)
    and a
    jr z,bomb_inactive
    ex de,hl
    ld b,(hl)
    ld a,b
    and #0x7E
    inc l
    add a,(hl)
    ld c,a
    and #0x1F
    ld (hl),a
    inc l
    ld a,c
    rlca
    rlca
    rlca
    and #7
    bit 7,b
    jr z,bomb_dx_ready
    neg
bomb_dx_ready:
    ex de,hl
    add a,(hl)
    ld (hl),a
    inc l
    ld a,(hl)
    add a,ixh
    ld (hl),a
    rra
    xor ixh
    rlca
    jr nc,bomb_advance
    ld h,#0x9B
    rrc (hl)
    ccf
    rl (hl)
bomb_advance:
    inc l
    dec ixl
    jr nz,bomb_loop
    ret
bomb_inactive:
    ld h,#0x88
    ld (hl),#0x80
    inc l
    inc e
    inc e
    jr bomb_advance

; CPU sub: sprite output and bomb generation, with rev. B CTRL write.
.org 0x0D03
l_0D03_flite_pth_posn_set:
       ld   a,(0x9215)
       ld   c,a

; extract x-coord and adjust for homing if needed
; fixed point 9.7 in .b02.b03 - left shift integer portion into A ... carry in to <0> from .b02<7>
       ld   h,#>0x9300
       ld   d,0x03(ix)                            ; bits<1:6> of x pixel ... could load directly to A
; set Cy from .b02<7>
       ld   a,#0x7F
       cp   0x02(ix)

       ld   a,d                                   ; (ix)0x03
       rla                                        ; rotate Cy into A<0> (.b02<7>)

       bit  0,c                                   ; test flip screen
       jr   z,l_0D1A
       add  a,#0x0D                               ; flipped
       cpl
l_0D1A:
       bit  6,0x13(ix)                            ; if !z, add  a,(ix)0x11 ... relative offset
       jr   z,l_0D23
       add  a,0x11(ix)                            ; heading home (add x-offset ... already bits<7:0>)
l_0D23:
       ld   (hl),a                                ; &sprite[n].posn.x

; extract y-coord and adjust for homing if needed
       inc  l                                     ; sprite[n].posn.sy<0:7>
       ld   b,0x01(ix)
; set Cy from .b00<7>
       ld   a,#0x7F
       cp   0x00(ix)

       rl   e                                     ; rotate Cy into E<0>
       ld   a,b                                   ; (ix)0x01

       bit  0,c                                   ; test flip screen
       jr   nz,l_0D38
       add  a,#(<(-0x0160 - 0x02))>>1             ; not flipped ... lsb of result, right-shift 1
       cpl
       dec  e                                     ; compliment bit-0 of 9-bit integer portion

l_0D38:
; E<0> <- Cy <- 7 6 5 4 3 2 1 0 <- Cy <- E<0>
       rr   e                                     ; bit-0 of 9-bit integer portion into Cy
       rla                                        ; Cy (bit0) into lsb, bit8 into Cy
       rl   e                                     ; bit8 from Cy into E<0> (bit8 of sprite_y)

       bit  6,0x13(ix)                            ; if !z, add  a,(ix)0x12
       jr   z,l_0D50
; r16.word += mctl_mpool[mpidx].b12
       add  a,0x12(ix)                            ; heading home (step y coord)
       ld   d,a                                   ; stash bits<7:0> of sum
       rra                                        ; somehow the rest of this propogates bit9 of the sum into E<0>
       xor  0x12(ix)
       rlca
       ld   a,d
       jr   nc,l_0D50
       inc  e

l_0D50:
       ld   (hl),a                                ; sprite[n].posn.sy<7:0>
       ld   h,#>0x9B00                    ; sprite[n].posn.sy<8>
       rrc  (hl)
       rrc  e
       rl   (hl)                                  ; sprite[n].posn.sy<8>


; Once the timer in $0E is reached, then check conditions to enable bomb drop.
; If bomb is disabled for any reason, the timer is restarted.
       dec  0x0E(ix)                              ; countdown to enable a bomb
       jp   nz,next__pool_idx

       srl  0x0F(ix)                              ; these bits enable bombing
       jp   nc,l_0DF5_next_superloop_and_reload_0E

       ld   a,0x01(ix)                            ; if .cy.pair.b1 > $4C
       cp   #152>>1                               ; 0x4C
       jp   c,l_0DF5_next_superloop_and_reload_0E

       ld   a,(0x9000 + 0x15)          ; f_1F04 ...fire button input
       and  a
       jp   z,l_0DF5_next_superloop_and_reload_0E

       ld   a,(0x92AC + 1)
       and  a
       jp   nz,l_0DF5_next_superloop_and_reload_0E

; check for available bomb ... bombs are rendered inactive at l_0815
       ex   de,hl                                 ; &sprite.ctrl[bmbr].b1 to DE ...
       ld   hl,#0x8800 + 0x68                     ; bomb0 object/index
       ld   b,#8                                  ; check 8 positions
l_0D82:
       ld   a,(hl)                                ; _objs[BOMB0].state
       cp   #0x80                                 ; INACTIVE
       jr   z,l_0D8D_got_a_bullet
       inc  l
       inc  l
       djnz l_0D82

       jr   l_0DF5_next_superloop_and_reload_0E

l_0D8D_got_a_bullet:
       ld   (hl),#6                               ; _objs[BOMB0].state = BOMB
       ld   h,#0x9B
       ld   (hl),#1
       push hl                                    ; &_objs[BOMB0 + n]
       ld   h,#>0x9300
       ld   d,h
       dec  e                                     ; ... E from &sprite.ctrl[L].b1
; sprite.posn[BOMB0 + n].b0 = sprite.posn[e].b0
       ld   a,(de)                                ; bomber.x
       ld   c,a
       ld   (hl),a                                ; bomb.x
; sprite.posn[BOMB0 + n].b1 = sprite.posn[e].b1 // y<7:0>
       inc  e
       inc  l
       ld   a,(de)                                ; bomber.y<7:0>
       ld   b,a
       ld   (hl),a                                ; bomb.y<7:0>
; sprite.ctrl[BOMB0 + n].b1 = sprite.ctrl[e].b1
       ld   h,#>0x9B00
       ld   d,h
       ld   a,(de)                                ; bomber.y<8> (in :0 ... ctrl in :1)
       rrc  (hl)                                  ; bomb.ctrl.b1<0> to Cy
       rrca                                       ; bomber.y<8> from A<0> to Cy
       rl   (hl)                                  ; sY<8> from Cy to bomb.ctrl.b1<0>
       rlca                                       ; restore A with sY<8> left in Cy
       rr   b                                     ; bomber.y<8:1>
; if (mrw_sprite.posn[bomber_idx].b0 > sprite.posn[FGHTR].b0)
       ld   a,(0x9300 + 0x62)             ; fighter.x
       sub  c                                     ; bomb.x
       push af                                    ; stash fighter.x - bomber.x
       jr   nc,l_0DB1                             ; if bomber.x > fighter.x ...
       neg                                        ; ... then dX = -dX
l_0DB1:
; dX passed to 0x0EAE()in hl16 ... lsb of &bomb.ctrl.b1 remaining in L is insignificant
       ld   h,a

       ld   a,(0x9215)
       and  a
       ld   a,#298 >> 1                           ; 0x95 ... 354-56
       jr   z,l_0DBC
       ld   a,#56 >> 1                            ; 0x1C ... inverted
l_0DBC:
       sub  b                                     ; bomber.y<8:1>
       jr   nc,l_0DC1                             ; if bomber.y > fighter.y ...
       neg
l_0DC1:
       call 0x0EAE                                ; HL = HL / A
       ld   b,h
       ld   c,l
       srl  h
       rr   l
       srl  h
       rr   l
       add  hl,bc
       srl  h
       rr   l
       srl  h
       rr   l

       ld   a,h
       and  a
       jr   nz,l_0DE0
       ld   a,l
       cp   #0x60
       jr   c,l_0DE2
l_0DE0:
       ld   a,#0x60
l_0DE2:
       ld   b,a

       pop  af                                    ; Cy from (fighter.x - bomber.x)
       rr   b
       pop  hl                                    ; sprt_mctl_objs[bomb]
       ld   a,l
       add  a,#8                                  ; IDX_BOMB will be index 0 thanks to and $0F
       and  #0x0F
       ld   hl,#0x92B0 + 0x00                     ; bomb x-coordinate structure ( 8 * 2 )
       add  a,l
       ld   l,a
       ld   (hl),b
       inc  hl
       ld   (hl),#0

l_0DF5_next_superloop_and_reload_0E:
       ld   a,(0x92E2 + 0x00)                     ; to $0E(ix) e.g. A==14 (set for each round ... bomb drop counter)
       ld   0x0E(ix),a                            ; 0x92E2[0] ... bomb drop counter

next__pool_idx:
       ld   hl,#0x9289                     ; -= 1 ... counts backwards
       dec  (hl)
       ret  z
; mctl_pool_idx += 1
       ld   de,#0x0014                            ; size of object-movement structure
       add  ix,de

       jp   0x08E4


; CPU main: delayed captured-fighter transfer in rev. B.
.org 0x19B2
f_19B2:
;  if ( flag  != 0 )  goto show_text
       ld   a,(0x928A + 0x04)      ; == 1 when boss connects with ship (2318)
       and  a
       jr   nz,l_19D2_fighter_captured

;  else if ( game_tmrs[1] | 0 == 0 )  goto erase_text
;   ... with A already 0, we use OR to check Z flag for zero count... which efficently loads our count to A at the same time!
       ld   hl,#0x92AC + 1
       or   (hl)
       jr   z,l_1A01_erase_text

;  else if ( game_tmrs[1] != 4 ) goto 19c7
       cp   #4
       jr   nz,l_19C7

;  timer--   .....decrement timer at 4. ... why? the count is set to 6 (19D7)
       dec  a
       ld   (hl),a
       ld   (0x9AA0 + 0x09),a                     ; game_tmrs[1] ... sound-fx count/enable registers
l_19C7:
       ld   a,(0x9820 +9)       ; bug_flite_que[ plyr.cboss_slot ].b0D = 4
       add  a,#0x0D
       ld   l,a
       ld   h,#>0x9100                 ; bug_flite_que[ plyr.cboss_slot ].b0D = 4
       ld   (hl),#4
       ret

l_19D2_fighter_captured:
; sets up a captured-fighter as an enemy object
       ld   c,#0x0A                               ; index of string
       rst  0x30                                  ; string_out_pe "FIGHTER CAPTURED"
       ld   a,#6                                  ; set time of countdown
       ld   (0x92AC + 1),a                 ; 6 ...time to show fighter-captured-text
       inc  a
       ld   (0x8B63),a
       xor  a
       ld   (0x928A + 0x01),a      ; 0
       ld   (0x928A + 0x04),a      ; 0 .... erase fighter-captured text
       ret

l_1A01_erase_text:
; check if text has been cleared yet?
;  if ( *82d1 == $24 ) goto $1a10
       ld   a,(0x8000 + 0x02C0 + 0x11)        ; 'I' of fighter captured
       cp   #0x24
       jr   z,l_1A10
;  clear "fighter captured" text
       ld   hl,#0x9300 + 0x62             ; ship (1) position
       ld   a,(0x9820 +8)
       and  #7                                    ; captured ships are in same order as bosses, from 8800-8807
       ld   e,a
       ld   d,h
       ld   a,(hl)                                ; get column of ship object e.g. ... A := *(9362)
       ld   (de),a                                ; get column of captured ship object e.g. (DE == 9302)
       ld   (hl),#0
; set row offset of captured ship object
       inc  l
       inc  e
       ld   a,(hl)
       ld   (de),a
; odd byte of sprite ctrl...
       ld   h,#>0x9B00
       ld   d,h
       ldd
       ldi

       ld   h,#>0x8B00
       ld   l,e
       ld   (hl),#7                               ; sprite code 7 is the vertical ship "wing closed" used for captured ship
       dec  l
       ld   (hl),#7                               ; sprite color ... red
       ld   c,#0x0B                               ; index into string table
       ld   hl,#0x8000 + 0x03A0 + 0x11        ; "leftmost" column of row where "fighter captured" is displayed
       call 0x13B3                          ; erase fighter capture text
l_1A10:
       ld   a,(0x9820 +8)
       ld   l,a
       and  #0x07                                 ; ships are in same order as bosses, from 8800-8807
       ld   e,a
       ld   h,#>0x8800
       ld   a,(0x9215)
       ld   c,a
       ld   a,(hl)                                ; e.g. HL==8832
       cp   #0x09                                 ; check if object status "flying"
       jr   nz,l_1A3F_join_ship_to_group          ; status changes to 2 (rotating) when boss reaches home positn
       ld   h,#>0x9300
       ld   d,h                                   ; DE == captd ship sprite posn
       ld   a,(hl)                                ; HL == boss posn, horizontal
       ld   (de),a
       inc  l
       inc  e
       ld   a,#0x10                               ; offset captured-ship vertically from flying boss
;  if ( !flip_screen ) goto 1a31
       bit  0,c                                   ; C == _flip_screen
       jr   z,l_1A31
       neg                                        ; offset negated for inverted screen.
l_1A31:
       ld   b,a                                   ; A == vertical offset of ship from boss
       add  a,(hl)                                ; HL == boss in sprite posn regs (odds...vertical posn)
       ld   (de),a                                ; ... DE == boss_posn + $10
       rra
       xor  b
       rlca
       and  #0x01
       ld   h,#>0x9B00                    ; .b1
       ld   d,h
       xor  (hl)
       ld   (de),a                                ; update ship sprite ctrl (e.g. 9B03)
       ret

; ...boss status e.g. 8830[n] == $02  (rotating into position)
l_1A3F_join_ship_to_group:
; if ( couonter > 0 ) goto 1A4B
       ld   hl,#0x928A + 0x01      ; counter while captured ship is joined with the collective
       ld   a,(hl)                                ; a=0
       and  a
       jr   nz,l_1A4B_test_positioning_timer
; else initialize_ship_sprite
       ld   d,#>0x8B00
       ld   a,#6                                  ; ship sprite code
       ld   (de),a
l_1A4B_test_positioning_timer:
       inc  (hl)                                  ; *( 0x928A + 1 )++
       cp   #0x24
       jr   z,l_1A6A_ship_in_position

; set position increment +1 for inverted screen, otherwise -1
       ld   b,#1
       bit  0,c                                   ; C == flip_screen
       jr   nz,l_1A58
       dec  b
       dec  b
l_1A58:
       ld   l,e
       inc  l                                     ; set vertical position (odd-byte)
       ld   h,#>0x9300
       ld   a,b
       add  a,(hl)
       ld   (hl),a
       rra
       xor  b
       rlca
       ret  nc
       ld   h,#>0x9B00
       ld   a,(hl)
       xor  #0x01
       ld   (hl),a
       ret

l_1A6A_ship_in_position:
       xor  a
       ld   (0x9000 + 0x11),a          ; 0: this task
       ld   (0x9AA0 + 0x09),a                     ; 0: sound-fx count/enable registers
       ld   d,#>0x8800
       inc  a
       ld   (de),a                                ; 1: 0x8800[n] (stand-by position)
       ld   (0x9820 +8),a   ; 1: invalidate the capture boss object (e.g. was $32)
       ld   (0x99B9 + 0x00),a          ; 1: when fighter on screen
       inc  a
       ld   (0x9200 + 0x13),a              ; 2: restart-stage flag (fighter captured)
       ret

;;=============================================================================

; CPU main: reserve-preserving stage tokens and reserve redraw tail.
.org 0x117F
c_new_level_tokens:
;  memset($8002,$24,$12)
       ld   hl,#0x8000 + 2                    ; second row from bottom at right
       ld   b,#0x12
token_clear_top:
       ld   a,(hl)
       cp   #0x4A
       jr   nc,token_clear_top_next
       ld   (hl),#0x24
token_clear_top_next:
       inc  l
       djnz token_clear_top
       ld   l,#0x22
       ld   b,#0x12
token_clear_bottom:
       ld   a,(hl)
       cp   #0x4A
       jr   nc,token_clear_bottom_next
       ld   (hl),#0x24
token_clear_bottom_next:
       inc  l
       djnz token_clear_bottom

       ld   a,(0x9820 +1)           ; c_new_level_tokens
       ld   b,#0                                  ; tmp_quotient 50
       ld   hl,#0x8000 + 1                    ; offset into tile ram


; stage_ctr/50 and stage_ctr%50 (by brute force!)

;  while ( tmp_stage_ctr >= 50 ) {
l_1194_while:
       cp   #50
       jr   c,l_119F
       sub  #50                                   ; tmp_modulus
       inc  b                                     ; tmp_quotient
; offset tileram ptr 2 columns to the left... *50 icons are 2 tiles wide
       inc  l
       inc  l
       jr   l_1194_while

l_119F:
       ex   de,hl                                 ; stash the tileram offset in DE
       ld   l,a                                   ; stage_ctr % 50

       ld   h,#0
       ld   a,#10
       call 0x1061                              ; HL = HL / 10
       ld   h,a                                   ; A = HL % 10 ... (still have L==HL/10)

       push hl                                    ; stack the quotient and mod10 result
       ex   de,hl

; now HL == tile_ram address  and  DE == div10 and mod10 result

; offset base pointer in HL by the nbr of tile columns needed:

; if ( A >= 5 )  { C = A = A - 4 }  ... if mod10 > 5 then only 1 tile for the 5, plus nbr of 1's
       cp   #5
       jr   c,l_11B1_add_total
       sub  #4

l_11B1_add_total:
       ld   c,a                                   ; nbr of columns for 5's and 1's (not including 50's)

; Add up the total additional columns needed for 10, 10, 20, 30, 40 in A.
; 'bit 0' catches the odd div10 result and sets A=2. Noteing the 40 needs 4 columns,
; the div10 result does the right thing for 0, 20, and 40.
       ld   a,e                                   ; div10result
       bit  0,a                                   ; even or odd
       jr   z,l_11B9
       ld   a,#2
l_11B9:
       add  a,c                                   ; nbr of additional tile columns
       rst  0x10                                  ; HL += A

; B == count of 50's markers, if any
       inc  b                                     ; pre-increment for djnz ... B is at least 1

l_11BC_loop:
       djnz l_11DC_show_50s_tokens
       pop  bc                                    ; pop the quotient and mod10 result
       ld   a,c                                   ; div10 result
       call c_11E3_show_tokens_1

; if ( mod10result < 5 )
       ld   a,b                                   ; mod10 result
       cp   #5
       jr   c,l_11D0_do_1s_tokens
; else
       ld   d,#0x36 + 2                           ; tile nbr of top of '5' token ... bottom tile of token is next tile nbr
       call c_build_token_1                       ; show the 5 token

; do 1's ... mod10result -= 5
       ld   a,b
       sub  #5

l_11D0_do_1s_tokens:
       ld   b,a                                   ; nbr of 1's tokens
       inc  b                                     ; pre-increment for djnz

l_11D2_show_1s_loop:
       djnz l_11D5_while
       jp   0x137E

l_11D5_while:
       ld   d,#0x36                               ; tile nbr of top of '1' token ... bottom tile of token is next tile nbr
       call c_build_token_1                       ; show the token
       jr   l_11D2_show_1s_loop

l_11DC_show_50s_tokens:
       ld   a,#4
       call c_11E9                                ; show the token
       jr   l_11BC_loop


;;=============================================================================
;; c_11E3_show_tokens_1()
;;  Description:
;;   Setup c_11E9 to display the 10's tokens.
;; IN:
;;   A = count of 10's tiles to show, i.e. stage % 50 / 10
;; OUT:
;;  ...
;;-----------------------------------------------------------------------------
c_11E3_show_tokens_1:

; branch off depending on div10result ... do nothing for 0
       and  a                                     ; A==div10 result
       ret  z
; handle 40's separately
       cp   #4
       jr   z,l_11F0_do_40s

c_11E9:
; get offset of of start of tiles for 10, 20, and 30 ... div10result * 4
       rlca
       rlca
       add  a,#0x36                               ; base offset of icon tiles
       ld   d,a
       jr   l_11FA_show_10s_20s_30s_50s

l_11F0_do_40s:
       ld   d,#0x36 + 4 * 3                       ; offset to 30's tiles group
       call c_build_token_1
       call c_build_token_2

       ld   d,#0x36 + 4                           ; offset to 10's tiles group

l_11FA_show_10s_20s_30s_50s:
       call c_build_token_1
       call c_build_token_2

       ret

;;=============================================================================
;; c_build_token()
;;  Description:
;;   wrapper for c_build_token_2 that handles timing and sound-effect
;; IN:
;;   D = offset of start of tile group for the token to display
;;   HL = base address in tileram
;; OUT:
;;   HL -= 1
;;-----------------------------------------------------------------------------
c_build_token_1:

; check sound_disable_flag
       ex   af,af'
       jr   c,l_1215_restore_A_and_continue
; put it back
       ex   af,af'
; set the delay count 8 ... preserve frame count
       ld   a,(0x92A0 + 0)
       add  a,#8
       ld   e,a
l_120B:
       ld   a,(0x92A0 + 0)
       sub  e
       jr   nz,l_120B

; actv_plyr_state[0x05]==0 for challenge stage ... 0 count/enable for stage tokens clicks
       ex   af,af'
       ld   (0x9AA0 + 0x15),a                     ; 2 ... sound-fx count/enable registers, clicks for stage tokens

l_1215_restore_A_and_continue:
       ex   af,af'

;;=============================================================================
;; c_build_token_2()
;;  Description:
;;   Display 2 tiles comprising a stage token icon.
;;
;;   Each call to this displays a top and bottom tile. 4 tile tokens require
;;   this function to be called twice. HL is decremented in order to leave the
;;   pointer at the next column to the left.
;;   Use color table 1 for 1's, 5's and 50's.
;;   These are situated in the tile order such that the $0C mask leaves the 8,
;;   thereby setting the Z flag.
;;   Use color table 2 for the others.
;;   This is a bit roundabout and slightly confusing, here is a table of the
;;   tile numbers to help visualize it:
;;
;;    1's   36 37         38 & 0C = 08
;;    5's   38 39         3A & 0C = 08
;;    10's  3A 3B 3C 3D   3C & 0C = 0C  ... 3E & 0C = 0C
;;    20's  3E 3F 40 41   40 & 0C = 00  ... 42 & 0C = 00
;;    30's  42 43 44 45   44 & 0C = 00  ... 46 & 0C = 00
;;    50's  46 47 48 49   48 & 0C = 08  ... 5A & 0C = 08
;;
;; IN:
;;   D = offset of start of tile group for the token to display
;;   HL = base address in tileram
;; OUT:
;;   HL -= 1
;;
;;-----------------------------------------------------------------------------
c_build_token_2:
       ld   (hl),d
       inc  d                                     ; next tile
       set  5,l                                   ; +=32 ... advance one row down
       ld   (hl),d

       inc  d
       set  2,h                                   ; +=$0400 ... colorram

; if ( D & $0C  > 8 ) { A = 2  else A = 1 }
       ld   a,d
       and  #0x0C
       cp   #8
       ld   a,#1
       jr   z,l_1228
       inc  a

; set the color codes, resetting the bits and updating HL as we go
l_1228:
       ld   (hl),a
       res  5,l
       ld   (hl),a
       res  2,h
       dec  l                                     ; offset tileram pointer 1 column to the right

       ret

;;=============================================================================
