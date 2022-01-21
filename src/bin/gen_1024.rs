use std::env;
use std::fs::File;
use std::marker::PhantomData;

use csv::Writer;

use sa2_piece_gen::emerald_manager::EmeraldManager;
use sa2_piece_gen::rng::Rng;
use sa2_piece_gen::stage_spec::StageSpec;
use sa2_piece_gen::{Platform, Pc, Gc};
use sa2_piece_gen::hint_lookup::HintLookup;

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

fn main() {
    let mut args = env::args().skip(1);
    let platform = args.next().unwrap();
    let input_filename = args.next().unwrap();
    let hints_filename = args.next().unwrap();
    let output_filename = args.next().unwrap();
    let pre_calls_opt = args.next()
        .map(|s| s.parse().unwrap());

    let input = File::open(input_filename).unwrap();
    let mut spec: StageSpec = serde_json::from_reader(input).unwrap();

    let lookup = HintLookup::from_path(hints_filename);

    if let Some(pre_calls) = pre_calls_opt {
        spec.pre_calls = pre_calls;
    }

    match platform.as_str() {
        "pc" => gen_1024::<Pc>(spec, lookup, output_filename),
        "gc" => gen_1024::<Gc>(spec, lookup, output_filename),
        _ => unimplemented!(),
    }
}

fn gen_1024<P>(spec: StageSpec, lookup: HintLookup, output_filename: String)
    where P: Platform,
{
    let mut csv_writer = Writer::from_path(output_filename).unwrap();

    let r_iter: RngIterator<P> = RngIterator::new(spec.pre_calls);

    for (idx, r) in r_iter.take(1024).enumerate() {
        let mut em = EmeraldManager::from_spec::<P>(spec.clone());
        em.r = r;
        em.gen_pieces::<P>();
        csv_writer.write_record(&[
            idx.to_string(),
            em.p1.id.to_string(),
            em.p2.id.to_string(),
            em.p3.id.to_string(),
            lookup.lookup_piece(em.p1.id).h1.clone(),
            lookup.lookup_piece(em.p2.id).h1.clone(),
            lookup.lookup_piece(em.p3.id).h1.clone(),
        ]).unwrap();
    }
}
