/* Verification harness only; never compiled into the Rust game. */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "galag.h"
void flv_init_data(void);
uint8 irq_acknowledge_enable_cpu0,irq_acknowledge_enable_cpu1,nmi_acknowledge_enable_cpu2;
unsigned char tile[1024],colour[1024],sp1[128],sp2[128],sp3[128],stars[6];
unsigned char *videoram=tile,*colorram=colour,*spriteram=sp1,*spriteram_2=sp2,*spriteram_3=sp3,*galaga_starcontrol=stars;
int _updatescreen(int a) { return 0; }
void c_io_cmd_wait(void) {}
void galaga_sound_w(int a,int b) {}
void f_2916(void);void f_1DE6(void);void f_2A90(void);void f_23DD(void);void f_1F85(void);void f_1F04(void);
uint64_t hash=14695981039346656037ULL;
void byte(uint8_t b){hash^=b;hash*=1099511628211ULL;}
void arithmetic(void);
int main(int argc, char **argv){
 if(argc>1){arithmetic();return 0;}
 cpu1_init();
 plyr_actv.stg_ct=1;plyr_actv.not_chllng_stg=2;plyr_actv.atkwv_enbl=1;
 glbls9200.glbl_enemy_enbl=3;
 gctl_stg_fmtn_hpos_init(0);gctl_stg_new_etypes_init();gctl_stg_new_atk_wavs_init();g_mssl_init();
 for(int i=0;i<128;i++)sprt_mctl_objs[i].state=INACTIVE;
 mrw_sprite.cclr[SPR_IDX_SHIP].b0=6;mrw_sprite.cclr[SPR_IDX_SHIP].b1=9;
 mrw_sprite.posn[SPR_IDX_SHIP].b0=0x7a;mrw_sprite.posn[SPR_IDX_SHIP].b1=0x29;mrw_sprite.ctrl[SPR_IDX_SHIP].b1=1;
 io_input[1]=0x3f;task_actv_tbl_0[8]=1;task_actv_tbl_0[10]=1;
 for(int frame=0;frame<4096;frame++){
  if(task_actv_tbl_0[8])f_2916();
  if(task_actv_tbl_0[9])f_1DE6();
  if(task_actv_tbl_0[10])f_2A90();
  f_23DD();f_1F85();
  cpu1_rst38();
  hash=14695981039346656037ULL;
  for(int i=0;i<128;i+=2){
   byte(sprt_mctl_objs[i].state);byte(mrw_sprite.cclr[i].b0);byte(mrw_sprite.cclr[i].b1);
   byte(mrw_sprite.posn[i].b0);byte(mrw_sprite.posn[i].b1);byte(mrw_sprite.ctrl[i].b1&1);byte(mrw_sprite.ctrl[i].b0);
  }
  for(int j=0;j<8;j++)putchar((hash>>(8*j))&255);
 }
 return 0;
}

void pengo_sound_w(int a,int b) {}

#include "gg1-5.c"
void word(uint16_t w){byte(w&255);byte(w>>8);}
void result(void){for(int j=0;j<8;j++)putchar((hash>>(8*j))&255);hash=14695981039346656037ULL;}
void arithmetic(void){
 for(unsigned d=0;d<256;d++)for(unsigned n=0;n<65536;n++)word(mctl_div_16_8(n,d));
 result();
 for(unsigned x=0;x<256;x++)for(unsigned y=0;y<256;y++){
  mctl_mpool[0].cx.word=x<<8;mctl_mpool[0].cy.word=y<<8;
  word(mctl_rotn_hp(0x4080,0));
 }
 result();
 for(unsigned angle=0;angle<1024;angle++)for(unsigned ds=0;ds<16;ds++)for(unsigned seed=0;seed<4;seed++){
  mctl_mpool[0].cx.word=seed*21845;mctl_mpool[0].cy.word=65535-seed*21845;mctl_mpool[0].ang.word=angle;
  mctl_coord_incr(ds,0);word(mctl_mpool[0].cx.word);word(mctl_mpool[0].cy.word);
 }
 result();
}
