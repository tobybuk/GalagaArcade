/* Independent secondary reference for the common rocket-hit paths only.
 * This calls the pinned, UNMODIFIED C translation. Known C errors (pending
 * notification bit handling, flying-boss attributes, and 16-bit counter borrow)
 * are checked against the assembly separately instead of accepted as a golden.
 */
#define main entrance_reference_main
#include "reference-harness.c"
#undef main

int main(void) {
    cpu1_init();
    glbls9200.glbl_enemy_enbl = 3;
    const unsigned char colours[5] = {2,3,2,3,0};
    const unsigned char states[5] = {1,1,3,3,1};
    for (unsigned scenario=0; scenario<5; scenario++) {
        hash=14695981039346656037ULL;
        for (unsigned x=0; x<256; x++) for (unsigned y=0; y<512; y++) {
            memset(sprt_mctl_objs,0x80,sizeof(sprt_mctl_obj_t)*128);
            memset(sprt_hit_notif,0,96);
            memset(&mrw_sprite,0,sizeof(mrw_sprite));
            memset(mctl_mpool,0,sizeof(mctl_pool_t)*12);
            memset(ds_bug_collsn,0,16);
            memset(b_9AA0,0,32);
            plyr_actv.hit_ct=0;
            plyr_actv.dblfghtr=0;
            w_bug_flying_hit_cnt=0;
            b_bug_flyng_hits_p_round=0;
            sprt_mctl_objs[8].state=states[scenario];
            sprt_mctl_objs[8].mctl_idx=0;
            mrw_sprite.cclr[8].b0=0x18;
            mrw_sprite.cclr[8].b1=colours[scenario];
            mrw_sprite.posn[8].b0=x;
            mrw_sprite.posn[8].b1=y;
            mrw_sprite.ctrl[8].b1=(y>>8)&1;
            mctl_mpool[0].b13=1;
            mrw_sprite.posn[0x64].b0=128;
            mrw_sprite.posn[0x64].b1=0;
            mrw_sprite.ctrl[0x64].b1=1;
            hitd_det_rckt(0x64,0,48);
            word(plyr_actv.hit_ct);
            byte(mrw_sprite.posn[0x64].b0);
            byte(mrw_sprite.posn[0x64].b1);
            byte(mrw_sprite.ctrl[0x64].b1&1);
            byte(mrw_sprite.cclr[8].b1);
            byte(sprt_hit_notif[8]);
            byte(mctl_mpool[0].b13);
            for(unsigned i=0;i<16;i++)byte(ds_bug_collsn[i]);
            for(unsigned i=0;i<8;i++)byte(b_9AA0[i]);
            byte(b_bug_flyng_hits_p_round);
            byte(w_bug_flying_hit_cnt&255);
        }
        result();
    }
    return 0;
}
