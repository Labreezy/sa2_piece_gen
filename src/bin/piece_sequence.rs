use std::env;
use std::fs::File;
use std::marker::PhantomData;

use sa2_piece_gen::rng::Rng;
use sa2_piece_gen::emerald_manager::EmeraldManager;
use sa2_piece_gen::stage_spec::StageSpec;
use sa2_piece_gen::{Platform, Pc, Gc};

struct RngIterator<P> {
    r: Rng,
    p: PhantomData<P>
}

impl<P> RngIterator<P> {
    fn new(p: u32) -> RngIterator<P>
        where P: Platform,
    {
        let mut r = Rng::new(0xDEAD0CAB);
        for _ in 0..p {
            r.gen_val::<P::Consts>();
        }
        RngIterator {
            r: r,
            p: PhantomData,
        }
    }
}

impl<P> Iterator for RngIterator<P>
    where P: Platform,
{
    type Item = Rng;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(self.r);
        self.r.gen_val::<P::Consts>();
        ret
    }
}

//fn main() {
//    let mut em = EmeraldManager::from_process("sonic2app.exe");
//    em.gen_pieces(0);
//}

fn main() {
    let mut args = env::args().skip(1);
    let platform = args.next().unwrap();
    let input_filename = args.next().unwrap();

    let input = File::open(input_filename).unwrap();
    let spec: StageSpec = serde_json::from_reader(input).unwrap();

    match platform.as_str() {
        "pc" => piece_sequence::<Pc>(spec),
        "gc" => piece_sequence::<Gc>(spec),
        _ => unimplemented!(),
    }
}

fn piece_sequence<P>(spec: StageSpec)
    where P: Platform,
{
    let r_iter: RngIterator<P> = RngIterator::new(0);

    for (idx, r) in r_iter.enumerate() {
        let r_copy = r;
        let mut em = EmeraldManager::from_spec::<P>(spec.clone());
        em.r = r;
        em.gen_pieces::<P>();
        //if em.p1.id == 0x0301 && em.p2.id == 0x0007 && em.p3.id == 0x0807 {
        if em.p1.id == 0x010C && em.p2.id == 0x0503 && em.p3.id == 0x0806 {
            println!("{:04}: {:04X} {:04X} {:04X}", idx, em.p1.id, em.p2.id, em.p3.id);
            println!("Rng state: 0x{:08X}", r_copy.get_state());
            println!();
        }
    }
}
