#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::env;

#[cfg(windows)]
use sa2_piece_gen::stage_spec::StageSpec;
#[cfg(windows)]
use sa2_piece_gen::Pc;

#[cfg(windows)]
fn main() {
    let output = env::args().skip(1).next().unwrap();
    let file = File::create(output).unwrap();
    let spec = StageSpec::from_process::<Pc>("sonic2app.exe");
    serde_json::to_writer_pretty(file, &spec).unwrap();
}

#[cfg(not(windows))]
fn main() {
}
