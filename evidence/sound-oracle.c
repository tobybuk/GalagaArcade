/* Isolated common sounds from the pinned, unmodified C research translation.
 * Revision-specific beam/ship-hit/challenge tails are deliberately separate.
 * No CPU emulator. This never links into the Rust/browser game.
 */
#define main entrance_reference_main
#define pengo_sound_w ignored_pengo_sound_w
#include "reference-harness.c"
#undef pengo_sound_w
#undef main
#include "gg1-7.c"

static uint8_t registers[32];
void pengo_sound_w(int offset,int data) {registers[offset&31]=data&15;}
int main(void) {
    const uint8_t sounds[]={1,2,3,4,8,9,10,11,12,13,15,16,17,19,20,21,22};
    for (unsigned scenario=0;scenario<sizeof(sounds);scenario++) {
        memset(b_9A00,0,sizeof(b_9A00));memset(b_9A30,0,sizeof(b_9A30));
        memset(b_9A60,0,sizeof(b_9A60));memset(b_9A70,0,sizeof(b_9A70));
        memset(b_9AA0,0,sizeof(b_9AA0));memset(b_9AC0,0,sizeof(b_9AC0));
        memset(b_9A80_8,0,sizeof(b_9A80_8));memset(b_9A70_5,0,sizeof(b_9A70_5));
        memset(registers,0,sizeof(registers));
        w_9A80_04=w_9A80_06=0;b_9A82_idx=b_9A80_00=b_9A80_01=0;
        glbls9200.formatn_mv_signage=0;b_9AA0[sounds[scenario]]=1;
        for (unsigned tick=0;tick<1024;tick++) {
            cpu2_NMI();hash=UINT64_C(14695981039346656037);
            for (unsigned i=0;i<48;i++) byte(b_9A00[i]);
            for (unsigned i=0;i<48;i++) byte(b_9A30[i]);
            for (unsigned i=0;i<32;i++) byte(b_9AA0[i]);
            for (unsigned i=0;i<32;i++) byte(b_9AC0[i]);
            /* The C aid's volume envelope uses stale A for CPL, omits SUB 10
             * before NEG, and reads an unaliased voice byte at completion.
             * Compare its independently computed frequency/wave output only;
             * Rust volume envelopes are checked against the supplied assembly.
             */
            for (unsigned i=0;i<32;i++) if (i!=0x15 && i!=0x1a && i!=0x1f) byte(registers[i]);
            result();
        }
    }
}
