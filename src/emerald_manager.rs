use std::cmp::Ordering;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

use sa2_set::{SetFile, Pc};
#[cfg(windows)]
use process_reader::ProcessHandle;

use crate::rng::Rng;
use crate::vector::Vector;
use crate::stage_spec::{Emerald, StageSpec};
use crate::Platform;

const NUM_RNG_CALLS: u32 = 138;

struct F32Cmp(f32);

impl PartialEq<F32Cmp> for F32Cmp {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for F32Cmp {}

impl PartialOrd<F32Cmp> for F32Cmp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for F32Cmp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone, Debug)]
pub struct EmeraldManager {
    pub slot1_pieces: Vec<Emerald>,
    pub slot2_pieces: Vec<Emerald>,
    pub slot3_pieces: Vec<Emerald>,
    pub enemy_pieces: Vec<Emerald>,
    pub p1: Emerald,
    pub p2: Emerald,
    pub p3: Emerald,
    pub r: Rng,
}

impl EmeraldManager {
    pub fn from_set_file<P, R>(read: R) -> io::Result<EmeraldManager>
        where P: Platform,
              R: Read,
    {
        let set_file = SetFile::from_read::<Pc, _>(read)?;

        let mut slot1_pieces = Vec::new();
        let mut slot2_pieces = Vec::new();
        let mut slot3_pieces = Vec::new();
        let mut enemy_pieces = Vec::new();

        for object in set_file.0 {
            if object.object.0 == 0x0F {
                match object.rotation.x & 0xFF00 {
                    0x0100 | 0x0300 => slot1_pieces.push(Emerald {
                        id: object.rotation.x,
                        position: Vector {
                            x: object.position.x,
                            y: object.position.y,
                            z: object.position.z,
                        }
                    }),
                    0x0000 | 0x0200 | 0x0500 => slot2_pieces.push(Emerald {
                        id: object.rotation.x,
                        position: Vector {
                            x: object.position.x,
                            y: object.position.y,
                            z: object.position.z,
                        }
                    }),
                    0x0400 | 0x0700 | 0x0800 => slot3_pieces.push(Emerald {
                        id: object.rotation.x,
                        position: Vector {
                            x: object.position.x,
                            y: object.position.y,
                            z: object.position.z,
                        }
                    }),
                    _ => {}
                }
            }
            if object.object.0 == 0x0038 ||
               object.object.0 == 0x003E ||
               object.object.0 == 0x003B
            {
                if object.rotation.y != 0x00FF {
                    enemy_pieces.push(Emerald {
                        id: 0x0A00 | object.rotation.y,
                        position: Vector {
                            x: object.position.x,
                            y: object.position.y,
                            z: object.position.z,
                        }
                    })
                }
            }
        }

        let mut r = Rng::new(0xDEAD0CAB);
        for _ in 0..NUM_RNG_CALLS {
            r.gen_val::<P::Consts>();
        }

        Ok(EmeraldManager {
            slot1_pieces: slot1_pieces,
            slot2_pieces: slot2_pieces,
            slot3_pieces: slot3_pieces,
            enemy_pieces: enemy_pieces,
            p1: Emerald::default(),
            p2: Emerald::default(),
            p3: Emerald::default(),
            r: r,
        })
    }

    #[cfg(windows)]
    pub fn from_process<P>(process_name: &str) -> EmeraldManager
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

        let mut r = Rng::new(0xDEAD0CAB);
        for _ in 0..NUM_RNG_CALLS {
            r.gen_val::<P::Consts>();
        }

        EmeraldManager {
            slot1_pieces: p1_list,
            slot2_pieces: p2_list,
            slot3_pieces: p3_list,
            enemy_pieces: en_list,
            p1: Emerald::default(),
            p2: Emerald::default(),
            p3: Emerald::default(),
            r: r,
        }
    }

    pub fn from_set_file_path<P, A>(path: A) -> io::Result<EmeraldManager>
        where P: Platform,
              A: AsRef<Path>,
    {
        let file = File::open(path)?;
        Self::from_set_file::<P, _>(file)
    }

    pub fn from_spec<P>(spec: StageSpec) -> EmeraldManager
        where P: Platform,
    {
        let mut r = Rng::new(0xDEAD0CAB);
        for _ in 0..spec.pre_calls {
            r.gen_val::<P::Consts>();
        }

        EmeraldManager {
            slot1_pieces: spec.slot1_pieces,
            slot2_pieces: spec.slot2_pieces,
            slot3_pieces: spec.slot3_pieces,
            enemy_pieces: spec.enemy_pieces,
            p1: Emerald::default(),
            p2: Emerald::default(),
            p3: Emerald::default(),
            r: r,
        }
    }

    pub fn gen_pieces<P>(&mut self)
        where P: Platform,
    {
        // Generate piece 1
        if self.p1.id != 0xFE00 {
            let num_p1 = self.slot1_pieces.len() + self.enemy_pieces.len();

            let p1_index = ((self.r.gen_val::<P::Consts>() as f32 / 32768.0) * num_p1 as f32) as usize;

            self.p1 = if p1_index < self.slot1_pieces.len() {
                self.slot1_pieces[p1_index]
            }
            else {
                self.enemy_pieces.swap_remove(p1_index - self.slot1_pieces.len())
            };
        }

        // Generate piece 2
        if self.p2.id != 0xFE00 {
            let mut potential_p2: Vec<_> = self.slot2_pieces.iter().chain(self.enemy_pieces.iter()).collect();

            potential_p2.sort_by_key(|p| F32Cmp(p.position.distance::<P::Math>(self.p1.position)));

            let num_p2 = self.slot2_pieces.len() + self.enemy_pieces.len();

            let mut p2_index = (num_p2 as f32 - (((self.r.gen_val::<P::Consts>() as f32 / 32768.0) * num_p2 as f32) / 2.0)) as usize;
            if p2_index >= num_p2 {
                p2_index -= 1;
            }

            self.p2 = *potential_p2[p2_index];
        }

        // Generate piece 3
        if self.p3.id != 0xFE00 {
            let mut potential_p3: Vec<_> = self.slot3_pieces.iter().collect();

            potential_p3.sort_by_key(|p| F32Cmp((p.position - self.p2.position).cross::<P::Math>(p.position - self.p1.position).magnitude::<P::Math>()));

            let num_p3 = self.slot3_pieces.len();
            let rand_val = self.r.gen_val::<P::Consts>();
            let mut p3_index = (num_p3 as f32 - (((rand_val as f32 / 32768.0) * num_p3 as f32) / 2.0)) as usize;

            if p3_index >= num_p3 {
                p3_index -= 1;
            }
            self.p3 = *potential_p3[p3_index];
        }
    }

    pub fn gen_pieces_full<P>(&mut self, frame: u32)
        where P: Platform,
    {
        for _ in 0..(frame % 1024) {
            self.r.gen_val::<P::Consts>();
        }
    }
}
