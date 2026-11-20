use galaga_reconstructed::{Object,starfield::{Star,Starfield},video};

#[test]
fn framebuffer_matches_independent_hardware_renderer_at_boundaries_and_control_combinations() {
    let reference=include_bytes!("../evidence/reference-video.bin");
    let mut image=vec![0;video::WIDTH*video::HEIGHT*4];
    for (case,expected) in reference.chunks_exact(8).enumerate() {
        let xs=[0,1,16,17,31,32,33,128,224,225,240,255];
        let ys=[0,1,24,39,40,41,128,255];
        let mut objects=[Object::default();64];
        objects[0]=Object {x:xs[case%12],y:ys[(case/12)%8],
            code:(case as u8).wrapping_mul(37)&127,colour:(case as u8).wrapping_mul(11)&63,
            flags:((case/64)&15) as u8,..Object::default()};
        objects[0].write_odd_control(((case/96)&3) as u8|((case as u8).wrapping_mul(29)&0xfc));
        let mut tiles=[0x24;1024];let mut colours=[3;1024];
        if case&1!=0 {
            for i in 0..1024 {tiles[i]=(i as u8).wrapping_mul(17).wrapping_add((case as u8).wrapping_mul(3));
                colours[i]=(i as u8).wrapping_mul(13).wrapping_add((case as u8).wrapping_mul(7));}
        }
        let mut stars=Starfield::default();
        stars.visible.push(Star {x:(40+case%200) as u16,y:(case%224) as u8,colour:(case&63) as u8});
        video::render(&objects,&tiles,&colours,&stars,(case>>9) as u8,&mut image);
        let hash=image.iter().fold(14695981039346656037u64,|hash,byte|(hash^*byte as u64).wrapping_mul(1099511628211));
        assert_eq!(hash,u64::from_le_bytes(expected.try_into().unwrap()),"hardware raster case {case}");
    }
}
