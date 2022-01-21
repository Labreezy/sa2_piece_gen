use std::env;
use std::u32;

use sa2_piece_gen::rng::Rng;
use sa2_piece_gen::{Platform, Pc, Gc};

fn main() {
    let mut args = env::args().skip(1);
    let platform = args.next().unwrap();
    let stop_state_string = args.next().unwrap();

    let stop_state = u32::from_str_radix(&stop_state_string, 16).unwrap();

    let count = match platform.as_str() {
        "pc" => reverse_rng::<Pc>(stop_state),
        "gc" => reverse_rng::<Gc>(stop_state),
        _ => unimplemented!(),
    };

    println!("{}", count);
}

fn reverse_rng<P>(stop_state: u32) -> u32
    where P: Platform,
{
    let mut r = Rng::new(0xDEAD0CAB);

    let mut count = 0;
    loop {
        if r.get_state() == stop_state {
            break;
        }
        r.gen_val::<P::Consts>();
        count += 1;
    }

    count
}
