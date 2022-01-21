use std::io::Cursor;
use std::fs::File;
use std::path::Path;

use sa2_text::{Sa2TextTable, Sa2Text, TextElement, Language};
use prs_util::decoder::Decoder;

trait Sa2TextExt {
    fn concat_text(&self) -> String;
}

impl Sa2TextExt for Sa2Text {
    fn concat_text(&self) -> String {
        let mut concated = String::new();
        for element in self.0.iter() {
            if let TextElement::Text(ref text) = element {
                concated += text;
            }
        }
        concated
    }
}

#[derive(Clone, Debug)]
pub struct Hint {
    pub h1: String,
    pub h2: String,
    pub h3: String,
}

#[derive(Clone, Debug)]
pub struct HintLookup {
    b_normal: Vec<Hint>,
    c_normal: Vec<Hint>,
    b_hidden: Vec<Hint>,
    c_hidden: Vec<Hint>,
    a_undergnd: Vec<Hint>,
    b_undergnd: Vec<Hint>,
    a_2p_undgnd: Vec<Hint>,
    a_pathmove: Vec<Hint>,
    a_1p_tech: Vec<Hint>,
    q_final: Vec<Hint>,
    q_inenemy: Vec<Hint>,
}

// 8
// 24
// 8
// 8
// 16
// 16
// 8
// 8
// 8
// 3
// 41
impl HintLookup {
    pub fn from_path<P>(path: P) -> HintLookup
        where P: AsRef<Path>,
    {
        let file = File::open(path).unwrap();
        let mut decoder = Decoder::new(file);
        let data = decoder.decode_to_vec().unwrap();
        let table = Sa2TextTable::from_seek(Cursor::new(data), Language::English).unwrap();
        let hints = table.texts
            .chunks(3)
            .map(|chunk| 
                Hint {
                    h1: chunk[0].concat_text(),
                    h2: chunk[1].concat_text(),
                    h3: chunk[2].concat_text(),
                }
            )
            .collect::<Vec<_>>();

        let b_normal = hints[0..8].to_vec();
        let c_normal = hints[8..32].to_vec();
        let b_hidden = hints[32..40].to_vec();
        let c_hidden = hints[40..48].to_vec();
        let a_undergnd = hints[48..64].to_vec();
        let b_undergnd = hints[64..80].to_vec();
        let a_2p_undgnd = hints[80..88].to_vec();
        let a_pathmove = hints[88..96].to_vec();
        let a_1p_tech = hints[96..104].to_vec();
        let q_final = hints[104..107].to_vec();
        let q_inenemy = hints[107..148].to_vec();

        HintLookup {
            b_normal: b_normal,
            c_normal: c_normal,
            b_hidden: b_hidden,
            c_hidden: c_hidden,
            a_undergnd: a_undergnd,
            b_undergnd: b_undergnd,
            a_2p_undgnd: a_2p_undgnd,
            a_pathmove: a_pathmove,
            a_1p_tech: a_1p_tech,
            q_final: q_final,
            q_inenemy: q_inenemy,
        }
    }

    pub fn lookup_piece(&self, id: u16) -> &Hint {
        let major_id = id >> 8;
        let minor_id = id & 0x00FF;
        match major_id {
            0x00 => &self.b_normal[minor_id as usize],
            0x01 => &self.c_normal[minor_id as usize],
            0x02 => &self.b_hidden[minor_id as usize],
            0x03 => &self.c_hidden[minor_id as usize],
            0x04 => &self.a_undergnd[minor_id as usize],
            0x05 => &self.b_undergnd[minor_id as usize],
            0x06 => &self.a_2p_undgnd[minor_id as usize],
            0x07 => &self.a_pathmove[minor_id as usize],
            0x08 => &self.a_1p_tech[minor_id as usize],
            0x09 => &self.q_final[minor_id as usize],
            0x0A => &self.q_inenemy[minor_id as usize],
            _ => panic!("Bad major id"),
        }
    }
}
