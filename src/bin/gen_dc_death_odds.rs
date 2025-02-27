use std::process::{Command, Stdio};
use std::fs::File;



fn main() {
    let prog_abspath = "D:\\rust_projects\\sa2_piece_gen\\target\\debug\\search_set.exe";
    let p1_id = "G0307";
    let file = File::create("0307.csv").unwrap();
    let stdio = Stdio::from(file);
    let search_dc_set = Command::new(prog_abspath).arg("-p").arg("pc").arg("-s").arg("D:\\rust_projects\\sa2_piece_gen\\spec_files\\PC\\dc_spec_pc.txt").arg("-e").arg("700000")
    .arg(p1_id).arg("X").arg("X")
    .stdout(stdio)
    .output().expect("failed set search");
    println!("DONE")

}