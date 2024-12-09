use std::env;
use std::error::Error;
use std::fs::File;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::process;
use std::u16;

use getopts::Options;

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
    DontCare,
}

impl PieceConstraint {
    fn from_str(s: &str) -> Result<PieceConstraint, ParseIntError> {
        if s.starts_with('G') {
            Ok(PieceConstraint::GrabbedId(u16::from_str_radix(s.strip_prefix('G').unwrap(), 16)?))
        }
        else if s == "X" {
            Ok(PieceConstraint::DontCare)
        }
        else {
            Ok(PieceConstraint::Want(u16::from_str_radix(&s, 16)?))
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage {} -p PLATFORM -s STAGE [OPTIONS] P1 P2 P3", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].as_str();

    let mut opts = Options::new();
    opts.reqopt("p", "platform", "set platform to simulate", "PLATFORM");
    opts.reqopt("s", "stage", "set stage-spec file", "STAGE");
    opts.optopt("b", "begin", "set initial RNG call amount (default 0)", "RNG_CALLS");
    opts.optopt("e", "end", "set final RNG call amount (default infinity)", "RNG_CALLS");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).expect("Could not parse arguments");

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let platform = matches.opt_str("p").unwrap();
    let input_filename = matches.opt_str("s").unwrap();
    let begin = matches.opt_get("b").expect("Error parsing begin value");
    let end = matches.opt_get("e").expect("Error parsing end value");

    if matches.free.len() != 3 {
        panic!("Incorrect number of piece descriptors (must be 3)");
    }

    let p1_string = &matches.free[0];
    let p2_string = &matches.free[1];
    let p3_string = &matches.free[2];

    let p1_id = PieceConstraint::from_str(p1_string).expect("Error parsing piece 1");
    let p2_id = PieceConstraint::from_str(p2_string).expect("Error parsing piece 2");
    let p3_id = PieceConstraint::from_str(p3_string).expect("Error parsing piece 3");

    let input = File::open(input_filename).expect("Error opening stage-spec file");
    let spec: StageSpec = serde_json::from_reader(input).expect("Error reading stage-spec file");

    match platform.as_str() {
        "pc" => piece_sequence::<Pc>(spec, begin, end, p1_id, p2_id, p3_id),
        "gc" => piece_sequence::<Gc>(spec, begin, end, p1_id, p2_id, p3_id),
        _ => unimplemented!(),
    }
}

fn piece_sequence<P>(spec: StageSpec, begin: Option<u32>, end: Option<u32>, p1: PieceConstraint, p2: PieceConstraint, p3: PieceConstraint)
    where P: Platform,
{
    let begin = begin.unwrap_or(0);
    let r_iter: RngIterator<P> = RngIterator::new(begin);

    for (idx, r) in r_iter.enumerate() {
        if let Some(end_val) = end {
            if begin + idx as u32 == end_val {
                return;
            }
        }

        let r_copy = r;
        let mut em = EmeraldManager::from_spec::<P>(spec.clone());

        match p1 {
            PieceConstraint::Want(id) => {
                if spec.get_emerald_by_id(id).is_none() {
                    panic!("Invalid p1 ID (not present in stage)");
                }
            },
            PieceConstraint::GrabbedId(id) => {
                em.p1 = spec.get_emerald_by_id(id).expect("Invalid p1 ID (not present in stage)");
                em.p1.id = 0xFE00;
            },
            _ => {}
        }

        match p2 {
            PieceConstraint::Want(id) => {
                if spec.get_emerald_by_id(id).is_none() {
                    panic!("Invalid p2 ID (not present in stage)");
                }
            },
            PieceConstraint::GrabbedId(id) => {
                em.p2 = spec.get_emerald_by_id(id).expect("Invalid p2 ID (not present in stage)");
                em.p2.id = 0xFE00;
            },
            _ => {}
        }

        match p3 {
            PieceConstraint::Want(id) => {
                if spec.get_emerald_by_id(id).is_none() {
                    panic!("Invalid p3 ID (not present in stage)");
                }
            },
            PieceConstraint::GrabbedId(id) => {
                em.p3 = spec.get_emerald_by_id(id).expect("Invalid p3 ID (not present in stage)");
                em.p3.id = 0xFE00;
            },
            _ => {}
        }

        em.r = r;
        em.gen_pieces::<P>();

        let mut matched = true;

        if let PieceConstraint::Want(id) = p1 {
            matched = matched && (em.p1.id == id);
        }

        if let PieceConstraint::Want(id) = p2 {
            matched = matched && (em.p2.id == id);
        }

        if let PieceConstraint::Want(id) = p3 {
            matched = matched && (em.p3.id == id);
        }

        if matched {
            println!("{}: {:04X} {:04X} {:04X}", begin + idx as u32, em.p1.id, em.p2.id, em.p3.id);
            println!("Rng state: 0x{:08X}", r_copy.get_state());
            println!();
        }
    }
}
