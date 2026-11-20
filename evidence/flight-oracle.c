/* Common sortie trajectories from the pinned UNMODIFIED C research aid.
 * Excludes capture/clone/FE handlers missing in that aid and compares only
 * the low active flag at termination, where C clears more bits than the ROM.
 * No bombing, formation movement or object-state task is enabled here.
 */
#define main entrance_reference_main
#include "reference-harness.c"
#undef main

int main(void) {
    const unsigned ids[4]={8,0x40,0x30,0x32};
    const unsigned paths[4]={0x34f,0x3a9,0x411,0x411};
    const unsigned colours[4]={3,2,0,0};
    cpu1_init();
    for(unsigned scenario=0;scenario<4;scenario++) {
        unsigned id=ids[scenario];
        memset(sprt_mctl_objs,0x80,sizeof(sprt_mctl_obj_t)*128);
        memset(&mrw_sprite,0,sizeof(mrw_sprite));
        memset(mctl_mpool,0,sizeof(mctl_pool_t)*12);
        memset(ds_new_stage_parms,0,11);
        memset(b_92C0_0,0,10);
        memset(ds4_game_tmrs,0,4);
        memset(task_actv_tbl_0,0,32);
        bmbr_cont_flag=0;
        b_92E2_stg_parm[0]=20;
        glbls9200.glbl_enemy_enbl=3;
        glbls9200.flip_screen=0;
        gctl_stg_fmtn_hpos_init(0);
        mrw_sprite.posn[0x62].b0=122;
        mrw_sprite.posn[id].b0=100;
        mrw_sprite.posn[id].b1=100;
        mrw_sprite.cclr[id].b0=6;
        mrw_sprite.cclr[id].b1=colours[scenario];
        bmbr_setup_fltq_boss(id | (scenario==3 ? 0x80 : 0),paths[scenario]);
        for(unsigned frame=0;frame<1024;frame++) {
            ds3_92A0_frame_cts[0]=frame;
            f_08D3();
            hash=14695981039346656037ULL;
            byte(sprt_mctl_objs[id].state);
            byte(mrw_sprite.cclr[id].b0);byte(mrw_sprite.cclr[id].b1);
            byte(mrw_sprite.posn[id].b0);byte(mrw_sprite.posn[id].b1);byte(mrw_sprite.ctrl[id].b1&1);byte(mrw_sprite.ctrl[id].b0);
            word(mctl_mpool[0].cy.word);word(mctl_mpool[0].cx.word);word(mctl_mpool[0].ang.word);
            word(mctl_mpool[0].p08.word);
            byte(mctl_mpool[0].b0D);byte(mctl_mpool[0].b0C);byte(mctl_mpool[0].b13&1);
            byte(mctl_mpool[0].b06);byte(mctl_mpool[0].b07);
            byte(mctl_mpool[0].b0A);byte(mctl_mpool[0].b0B);
            byte(mctl_mpool[0].b0E);byte(mctl_mpool[0].b0F);
            result();
        }
    }
}
