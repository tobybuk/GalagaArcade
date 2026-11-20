/* Independent hardware-reference renderer, used only to generate test evidence.
 * Wiring/layout from MAME galaga.cpp (Nicola Salmoria and contributors,
 * BSD-3-Clause, notice in ../NOTICE.md). No gameplay, CPU or Rust calls.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

static uint8_t characters[4096],sprites[8192],palette[32],character_colours[256],sprite_colours[256];
static uint8_t pixels[224*288*4],tiles[1024],colours[1024];
static void load(const char *root,const char *name,uint8_t *data,size_t length) {
    char path[4096];snprintf(path,sizeof(path),"%s/%s",root,name);
    FILE *f=fopen(path,"rb");if (!f || fread(data,1,length,f)!=length) exit(2);fclose(f);
}
static int bit(const uint8_t *data,int index) {return !!(data[index>>3]&(128>>(index&7)));}
static void colour(int index,uint8_t *rgb) {
    int p=palette[index&31];
    for (int c=0;c<2;c++) {
        int v=p>>(c*3);
        double sum=(v&1)/1000.0+((v>>1)&1)/470.0+((v>>2)&1)/220.0;
        rgb[c]=(uint8_t)floor(255.0*sum/(1.0/1000+1.0/470+1.0/220)+0.5);
    }
    rgb[2]=(uint8_t)floor(255.0*(((p>>6)&1)/470.0+((p>>7)&1)/220.0)/(1.0/470+1.0/220)+0.5);
}
static void put(int x,int y,const uint8_t *rgb) {
    if (x<0 || x>=224 || y<0 || y>=288) return;
    memcpy(pixels+(y*224+x)*4,rgb,3);
}
static void draw_star(int test) {
    int p=test&63;uint8_t rgb[3];
    for (int c=0;c<3;c++) {
        double sum=((p>>(c*2))&1)/470.0+((p>>(c*2+1))&1)/220.0;
        double total=1.0/470+1.0/220+(c<2 ? 1.0/1000 : 0);
        rgb[c]=(uint8_t)floor(255.0*sum/total+0.5);
    }
    put(223-test%224,40+test%200,rgb);
}
static void draw_sprite(int test) {
    static const int xs[]={0,1,16,17,31,32,33,128,224,225,240,255};
    static const int ys[]={0,1,24,39,40,41,128,255};
    int flags=(test/64)&15,flip=test>>9;
    int sx=ys[(test/12)%8]-40+256*((test/96)&3);
    int sy=((257-xs[test%12]-16*((flags>>3)&1))&255)-32;
    for (int by=0;by<=((flags>>3)&1);by++) for (int bx=0;bx<=((flags>>2)&1);bx++) {
        int fx=(flags&1)^flip,fy=((flags>>1)&1)^flip;
        int code=((test*37)&127)+(by^(((flags>>3)&1)*fy))*2+(bx^(((flags>>2)&1)*fx));
        for (int y=0;y<16;y++) for (int x=0;x<16;x++) {
            int xx=fx?15-x:x,yy=fy?15-y:y;
            int address=(code&127)*512+(xx/4)*64+(xx%4)+(yy%8)*8+(yy/8)*256;
            int pen=bit(sprites,address)*2+bit(sprites,address+4);
            int index=sprite_colours[((test*11)&63)*4+pen]&15;
            uint8_t rgb[3];colour(index,rgb);
            if (index!=15) put(223-(sy+16*by+y),sx+16*bx+x,rgb);
        }
    }
}
static void draw_tiles(int test) {
    int flip=test>>9;
    for (int row=0;row<28;row++) for (int col=0;col<36;col++) {
        int r=row+2,c=(col-2)&255;
        int offset=(c&32)?r+((c&31)<<5):c+(r<<5);
        for (int y=0;y<8;y++) for (int x=0;x<8;x++) {
            int address=(tiles[offset]&127)*128+(x<4?64+x:x-4)+8*y;
            int pen=bit(characters,address)*2+bit(characters,address+4);
            int index=16|(character_colours[(colours[offset]&63)*4+pen]&15);
            int px=223-(row*8+y),py=col*8+x;
            uint8_t rgb[3];colour(index,rgb);
            if (index!=31) put(flip?223-px:px,flip?287-py:py,rgb);
        }
    }
}
int main(int argc,char **argv) {
    if (argc!=2) return 2;
    load(argv[1],"characters.bin",characters,sizeof(characters));load(argv[1],"sprites.bin",sprites,sizeof(sprites));
    load(argv[1],"palette.bin",palette,sizeof(palette));load(argv[1],"character-colours.bin",character_colours,sizeof(character_colours));
    load(argv[1],"sprite-colours.bin",sprite_colours,sizeof(sprite_colours));
    for (int test=0;test<1024;test++) {
        memset(pixels,0,sizeof(pixels));for (int i=3;i<sizeof(pixels);i+=4) pixels[i]=255;
        for (int i=0;i<1024;i++) {tiles[i]=(test&1)?i*17+test*3:0x24;colours[i]=(test&1)?i*13+test*7:3;}
        draw_star(test);draw_sprite(test);draw_tiles(test);
        uint64_t hash=UINT64_C(14695981039346656037);
        for (int i=0;i<sizeof(pixels);i++) hash=(hash^pixels[i])*UINT64_C(1099511628211);
        for (int i=0;i<8;i++) putchar((uint8_t)(hash>>(8*i)));
    }
}
