use std::env;
use std::fs::File;
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::str::FromStr;
use std::u16;

use getopts::Options;

use sa2_piece_gen::hint_lookup::HintLookup;
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
    println!("{}", opts.usage(&brief));
    println!();
    println!("Pieces must be in hexadecimal format, major ID first.");
    println!();
    println!("Piece ID format (using 0x0A03 as an example):");
    println!("0A03    Find a set that has piece 0x0A03 in that slot");
    println!("G0A03   Mark that a given slot had piece 0x0A03 grabbed in the previous life");
    println!("X       Don't care. Any piece may show up and it counts as a match");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].as_str();

    let mut opts = Options::new();
    opts.optopt("p", "platform", "set platform to simulate", "PLATFORM");
    opts.optopt("s", "stage", "set stage-spec file", "STAGE");
    opts.optopt("b", "begin", "set initial RNG call amount (default 0)", "RNG_CALLS");
    opts.optopt("e", "end", "set final RNG call amount (default infinity)", "RNG_CALLS");
    opts.optopt("l", "lookup", "include hints with this PRS file in output", "ehxxxxe.PRS");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).expect("Could not parse arguments");

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let mut lookup = None;
    if matches.opt_present("l"){
        lookup = Some(HintLookup::from_path(matches.opt_str("l").unwrap()));
        
    }

    let platform = matches.opt_str("p").expect("Option missing: Platform (-p)");
    let input_filename = matches.opt_str("s").expect("Option missing: Stage (-s)");
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
        "pc" => piece_sequence::<Pc>(spec, begin, end, p1_id, p2_id, p3_id, lookup),
        "gc" => piece_sequence::<Gc>(spec, begin, end, p1_id, p2_id, p3_id, lookup),
        _ => unimplemented!(),
    }
}

fn piece_sequence<P>(spec: StageSpec, begin: Option<u32>, end: Option<u32>, p1: PieceConstraint, p2: PieceConstraint, p3: PieceConstraint, lookup: Option<HintLookup>)
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
            if let Some(ref hints) = lookup {
                let mut p1_hint = String::from_str("N/A").ok().unwrap();
                let mut p2_hint = String::from_str("N/A").ok().unwrap();
                let mut p3_hint = String::from_str("N/A").ok().unwrap();
                if em.p1.id != 0xFE00 {
                    p1_hint = hints.lookup_piece(em.p1.id).h1.replace("\n", " ");
                }
                if em.p2.id != 0xFE00 {
                    p2_hint = hints.lookup_piece(em.p2.id).h1.replace("\n", " ");
                }
                if em.p3.id != 0xFE00 {
                    p3_hint = hints.lookup_piece(em.p3.id).h1.replace("\n", " ");
                }
                println!("{}\t{:04X}\t{:04X}\t{:04X}\t{}\t{}\t{}", begin + idx as u32, em.p1.id, em.p2.id, em.p3.id, p1_hint, p2_hint, p3_hint); 
                
            } else {
                println!("{},{:04X},{:04X},{:04X}", begin + idx as u32, em.p1.id, em.p2.id, em.p3.id);
            }
        }
    }
}
