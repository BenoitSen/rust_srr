use core::panic;
use std::{
    fs::{self, File},
    io::Read,
};

#[derive(Debug)]
pub struct SrrFile {
    pub application_name: String,
    pub body: Vec<SrrBody>,
}

#[derive(Debug)]
pub struct SrrFileHeader {}

#[derive(Debug)]
pub struct SrrBody {}

#[derive(Debug)]
pub struct SrrBlock {
    head_crc: u16,
    head_type: HeadType,
    _head_flags: u16,
    head_size: u16,
    _add_size: Option<u32>,
}
impl SrrBlock {
    fn read_block_header(buffer: [u8; 7]) -> Self {
        println!("Block header : {:x?}", buffer);
        SrrBlock {
            head_crc: ((buffer[1] as u16) << 8) | buffer[0] as u16,
            head_type: HeadType::from_u8(buffer[2]),
            _head_flags: ((buffer[4] as u16) << 8) | buffer[3] as u16,
            head_size: ((buffer[6] as u16) << 8) | buffer[5] as u16,
            _add_size: None,
        }
    }

    fn read_header_block(&self, buffer: &[u8]) -> String {
        if self.head_type != HeadType::HeaderBlock {
            panic!("Srr block is not a Header Block");
        }

        if self.head_crc != 0x6969 {
            panic!("Header Block CRC is invalid : {}", self.head_crc);
        }

        let app_name_size = ((buffer[1] as u16) << 8 | buffer[0] as u16) as usize;
        let mut app_name = String::from("");
        if app_name_size > 0 {
            app_name = String::from_utf8(buffer[2..2 + app_name_size].to_vec()).unwrap();
        };

        app_name
    }
}

#[derive(Debug, PartialEq)]
enum HeadType {
    HeaderBlock,
    StoredFileBlock,
    OsoHashBlock,
    RarPadding,
    RarFileBlock,
    UnknownHeader,
}
impl HeadType {
    fn from_u8(head_type: u8) -> Self {
        match head_type {
            0x69 => HeadType::HeaderBlock,
            0x6A => HeadType::StoredFileBlock,
            0x6B => HeadType::OsoHashBlock,
            0x6C => HeadType::RarPadding,
            0x71 => HeadType::RarFileBlock,

            _ => HeadType::UnknownHeader,
        }
    }
}

impl SrrFile {
    pub fn from_file(filename: &str) -> Self {
        let mut f = File::open(filename).expect("no file found");
        let metadata = fs::metadata(filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        if f.read(&mut buffer).expect("buffer overflow") < 7 {
            panic!("File size is incoherent !");
        }

        let header_block = SrrBlock::read_block_header(buffer[0..7].try_into().unwrap());
        let application_name =
            header_block.read_header_block(&buffer[7..header_block.head_size as usize]);

        SrrFile {
            application_name,
            body: vec![],
        }
    }
}
