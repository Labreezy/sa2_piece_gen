use std::fs::File;
use std::env;

use sa2_piece_gen::stage_spec::StageSpec;
use sa2_piece_gen::Pc;

fn main() {
    let output = env::args().skip(1).next().unwrap();
    let file = File::create(output).unwrap();
    let spec = StageSpec::from_process::<Pc>("sonic2app.exe");
    serde_json::to_writer_pretty(file, &spec).unwrap();
}
