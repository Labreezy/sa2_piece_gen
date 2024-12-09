use std::env;
use std::fs::File;
use std::marker::PhantomData;
use std::u16;

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

enum PieceConstraint {
    Want(u16),
    GrabbedId(u16),
}

fn main() {
    let mut args = env::args().skip(1);
    let platform = args.next().unwrap();
    let input_filename = args.next().unwrap();
    let p1_string = args.next().unwrap();
    let p2_string = args.next().unwrap();
    let p3_string = args.next().unwrap();

    let p1_id = if p1_string.starts_with('G') {
        PieceConstraint::GrabbedId(u16::from_str_radix(p1_string.strip_prefix('G').unwrap(), 16).unwrap())
    }
    else {
        PieceConstraint::Want(u16::from_str_radix(&p1_string, 16).unwrap())
    };

    let p2_id = if p2_string.starts_with('G') {
        PieceConstraint::GrabbedId(u16::from_str_radix(p2_string.strip_prefix('G').unwrap(), 16).unwrap())
    }
    else {
        PieceConstraint::Want(u16::from_str_radix(&p2_string, 16).unwrap())
    };

    let p3_id = if p3_string.starts_with('G') {
        PieceConstraint::GrabbedId(u16::from_str_radix(p3_string.strip_prefix('G').unwrap(), 16).unwrap())
    }
    else {
        PieceConstraint::Want(u16::from_str_radix(&p3_string, 16).unwrap())
    };

    let input = File::open(input_filename).unwrap();
    let spec: StageSpec = serde_json::from_reader(input).unwrap();

    match platform.as_str() {
        "pc" => piece_sequence::<Pc>(spec, p1_id, p2_id, p3_id),
        "gc" => piece_sequence::<Gc>(spec, p1_id, p2_id, p3_id),
        _ => unimplemented!(),
    }
}

fn piece_sequence<P>(spec: StageSpec, p1: PieceConstraint, p2: PieceConstraint, p3: PieceConstraint)
    where P: Platform,
{
    let r_iter: RngIterator<P> = RngIterator::new(0);

//    match p1 {
//        PieceConstraint::Want(id) => println!("{:#?}", spec.get_emerald_by_id(id)),
//        PieceConstraint::GrabbedId(id) => println!("{:#?}", spec.get_emerald_by_id(id)),
//        _ => unimplemented!(),
//    }

    for (idx, r) in r_iter.enumerate() {
        let r_copy = r;
        let mut em = EmeraldManager::from_spec::<P>(spec.clone());

        let p1_id = match p1 {
            PieceConstraint::Want(id) => id,
            PieceConstraint::GrabbedId(id) => {
                em.p1 = spec.get_emerald_by_id(id).unwrap();
                em.p1.id = 0xFE00;
                0xFE00
            }
        };

        let p2_id = match p2 {
            PieceConstraint::Want(id) => id,
            PieceConstraint::GrabbedId(id) => {
                em.p2 = spec.get_emerald_by_id(id).unwrap();
                em.p2.id = 0xFE00;
                0xFE00
            }
        };

        let p3_id = match p3 {
            PieceConstraint::Want(id) => id,
            PieceConstraint::GrabbedId(id) => {
                em.p3 = spec.get_emerald_by_id(id).unwrap();
                em.p3.id = 0xFE00;
                0xFE00
            }
        };

        em.r = r;
        em.gen_pieces::<P>();

//        println!("{:#?}", em);
//        println!("{:#?}", r_copy);

        if em.p1.id == p1_id && em.p2.id == p2_id && em.p3.id == p3_id {
            println!("{:04}: {:04X} {:04X} {:04X}", idx, em.p1.id, em.p2.id, em.p3.id);
            println!("Rng state: 0x{:08X}", r_copy.get_state());
            println!();
        }
    }
}
