use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use serde_derive::{Serialize, Deserialize};
#[cfg(windows)]
use process_reader::ProcessHandle;
use byteorder::{ReadBytesExt, BE};

use crate::vector::Vector;
use crate::rng::Rng;
use crate::Platform;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Emerald {
    pub id: u16,
    pub position: Vector,
}

impl Default for Emerald {
    fn default() -> Emerald {
        Emerald {
            id: 0xFF00,
            position: Vector::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageSpec {
    pub slot1_pieces: Vec<Emerald>,
    pub slot2_pieces: Vec<Emerald>,
    pub slot3_pieces: Vec<Emerald>,
    pub enemy_pieces: Vec<Emerald>,
    pub pre_calls: u32,
}

impl StageSpec {
    #[cfg(windows)]
    pub fn from_process<P>(process_name: &str) -> StageSpec
        where P: Platform,
    {
        let p_handle = ProcessHandle::from_name_filter(|s| s.to_lowercase() == process_name).unwrap().unwrap();
        let em_addr = p_handle.read_u32(0x01AF014C).unwrap() as u64;
        let num_p1 = p_handle.read_u8(em_addr + 6).unwrap();
        let num_p2 = p_handle.read_u8(em_addr + 7).unwrap();
        let num_p3 = p_handle.read_u8(em_addr + 8).unwrap();
        let num_en = p_handle.read_u8(em_addr + 9).unwrap();

        let read_list = |addr, num| {
            let mut pieces = Vec::new();
            let mut addr = p_handle.read_u32(addr).unwrap() as u64;

            for _ in 0..num {
                let major_id = p_handle.read_u8(addr).unwrap();
                let minor_id = p_handle.read_u8(addr + 1).unwrap();
                let x = p_handle.read_f32(addr + 4).unwrap();
                let y = p_handle.read_f32(addr + 8).unwrap();
                let z = p_handle.read_f32(addr + 12).unwrap();
                pieces.push(Emerald {
                    id: (major_id as u16) << 8 | minor_id as u16,
                    position: Vector {
                        x: x,
                        y: y,
                        z: z,
                    }
                });
                addr += 16;
            }

            pieces
        };

        let p1_list = read_list(em_addr + 0x5C, num_p1);
        let p2_list = read_list(em_addr + 0x60, num_p2);
        let p3_list = read_list(em_addr + 0x64, num_p3);
        let en_list = read_list(em_addr + 0x68, num_en);

        let rng_state = p_handle.read_u32(0x05CE05BC).unwrap();
        let mut r = Rng::new(0xDEAD0CAB);
        let mut calls = 0;
        while r.get_state() != rng_state {
            calls += 1;
            r.gen_val::<P::Consts>();
        }

        StageSpec {
            slot1_pieces: p1_list,
            slot2_pieces: p2_list,
            slot3_pieces: p3_list,
            enemy_pieces: en_list,
            pre_calls: calls,
        }
    }

    pub fn from_path<P, A>(filename: A) -> StageSpec
        where P: Platform,
              A: AsRef<Path>,
    {
        let mut file = File::open(filename).unwrap();

        file.seek(SeekFrom::Start(0x00C5D5A6)).unwrap();
        let num_p1 = file.read_u8().unwrap();
        let num_p2 = file.read_u8().unwrap();
        let num_p3 = file.read_u8().unwrap();
        let num_en = file.read_u8().unwrap();

        file.seek(SeekFrom::Start(0x00C5D5FC)).unwrap();
        let p1_addr = file.read_u32::<BE>().unwrap() ^ 0x80000000;
        let p2_addr = file.read_u32::<BE>().unwrap() ^ 0x80000000;
        let p3_addr = file.read_u32::<BE>().unwrap() ^ 0x80000000;
        let en_addr = file.read_u32::<BE>().unwrap() ^ 0x80000000;

        file.seek(SeekFrom::Start(0x003AD6A0)).unwrap();
        let rng_state = file.read_u32::<BE>().unwrap();

        let mut read_list = |addr, num| {
            let mut pieces = Vec::new();
            file.seek(SeekFrom::Start(addr)).unwrap();

            for _ in 0..num {
                let id = file.read_u16::<BE>().unwrap();
                let _padding = file.read_u16::<BE>().unwrap();
                let x = file.read_f32::<BE>().unwrap();
                let y = file.read_f32::<BE>().unwrap();
                let z = file.read_f32::<BE>().unwrap();
                pieces.push(Emerald {
                    id: id,
                    position: Vector {
                        x: x,
                        y: y,
                        z: z,
                    }
                });
            }

            pieces
        };
    
        let p1_list = read_list(p1_addr as u64, num_p1);
        let p2_list = read_list(p2_addr as u64, num_p2);
        let p3_list = read_list(p3_addr as u64, num_p3);
        let en_list = read_list(en_addr as u64, num_en);

        let mut r = Rng::new(0xDEAD0CAB);
        let mut calls = 0;
        while r.get_state() != rng_state {
            calls += 1;
            r.gen_val::<P::Consts>();
        }

        StageSpec {
            slot1_pieces: p1_list,
            slot2_pieces: p2_list,
            slot3_pieces: p3_list,
            enemy_pieces: en_list,
            pre_calls: calls,
        }
    }

    pub fn from_set_bin<A>(filename: A) -> StageSpec
        where A: AsRef<Path>
        {
            let mut file = File::open(filename).unwrap();
            let n_objects = file.read_u32::<BE>().unwrap();
            file.seek(SeekFrom::Start(0x20)).unwrap();
            let mut p1_list: Vec<Emerald> = Vec::new();
            let mut p2_list: Vec<Emerald> = Vec::new();
            let mut p3_list: Vec<Emerald> = Vec::new();
            let mut pe_list: Vec<Emerald> = Vec::new();
            for i in 0..n_objects {
                let object_pos: u32 = (i+1)*0x20;
                println!("Object start: {object_pos:X}");
                file.seek(SeekFrom::Start(object_pos.into())).unwrap();
                let object_id = file.read_u16::<BE>().unwrap();
                if object_id != 0xF {
                    continue;
                }
                
                
                let major_id = file.read_u8().unwrap();
                let minor_id = file.read_u8().unwrap();
                let piece_id: u16 = (u16::from(major_id) << 8) + u16::from(minor_id);
                file.seek(SeekFrom::Current(4)).unwrap();
                let xpos = file.read_f32::<BE>().unwrap();
                let ypos = file.read_f32::<BE>().unwrap();
                let zpos = file.read_f32::<BE>().unwrap();
                
                
                let em_obj = Emerald {
                    id: piece_id,
                    position: Vector {
                        x: xpos,
                        y: ypos,
                        z: zpos
                    }
                };
                match major_id {
                    0xA => pe_list.push(em_obj),
                    4 | 7 | 8 => p3_list.push(em_obj),
                    0 | 2 | 5 => p2_list.push(em_obj),
                    1 | 3 => p1_list.push(em_obj),
                    _ => {
                        let seek_pos = file.stream_position().unwrap();
                        println!("Position: {seek_pos:X}");
                        println!("{major_id:X}");
                    }
                }

            }
            StageSpec {
                slot1_pieces: p1_list,
                slot2_pieces: p2_list,
                slot3_pieces: p3_list,
                enemy_pieces: pe_list,
                pre_calls: 136
            }
        }
    pub fn get_emerald_by_id(&self, id: u16) -> Option<Emerald> {
        for piece in &self.slot1_pieces {
            if piece.id == id {
                return Some(*piece);
            }
        }

        for piece in &self.slot2_pieces {
            if piece.id == id {
                return Some(*piece);
            }
        }

        for piece in &self.slot3_pieces {
            if piece.id == id {
                return Some(*piece);
            }
        }

        for piece in &self.enemy_pieces {
            if piece.id == id {
                return Some(*piece);
            }
        }

        None
    }
}
