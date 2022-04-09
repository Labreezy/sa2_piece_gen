use std::fs::File;
use std::env;

use sa2_piece_gen::stage_spec::StageSpec;
use sa2_piece_gen::Gc;

fn main() {
    let mut args = env::args().skip(1);
    let input = args.next().unwrap();
    let output = args.next().unwrap();

    let file = File::create(output).unwrap();
    let spec = StageSpec::from_path::<Gc, _>(input);
    serde_json::to_writer_pretty(file, &spec).unwrap();
}
