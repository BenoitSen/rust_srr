use core::panic;
use std::{
    fmt::Display,
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

pub struct SrrBlockHeader {
    head_crc: u16,
    head_type: HeadType,
    head_flags: u16,
    head_size: u16,
}
impl Display for SrrBlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Head CRC : {:x} - ", self.head_crc)?;
        write!(f, "Head Type : {:?} - ", self.head_type)?;
        write!(f, "Head Flags : {:x} - ", self.head_flags)?;
        write!(f, "Head Size : {:x}", self.head_size)
    }
}
impl SrrBlockHeader {
    fn read_block_header(buffer: &[u8]) -> Self {
        let block_header = SrrBlockHeader {
            head_crc: ((buffer[1] as u16) << 8) | buffer[0] as u16,
            head_type: HeadType::from_u8(buffer[2]),
            head_flags: ((buffer[4] as u16) << 8) | buffer[3] as u16,
            head_size: ((buffer[6] as u16) << 8) | buffer[5] as u16,
        };
        println!("---- Start block ----");
        println!("block header: {}", block_header);
        block_header
    }
}

#[derive(Debug)]
pub struct SrrBlock {}
impl SrrBlock {
    fn read_header_block(block_header: &SrrBlockHeader, buffer: &[u8]) -> String {
        if block_header.head_type != HeadType::HeaderBlock {
            panic!("Srr block is not a Header Block");
        }

        if block_header.head_crc != 0x6969 {
            panic!("Header Block CRC is invalid : {}", block_header.head_crc);
        }

        let app_name_size = ((buffer[1] as u16) << 8 | buffer[0] as u16) as usize;
        let mut app_name = String::from("");
        if app_name_size > 0 {
            app_name = String::from_utf8(buffer[2..2 + app_name_size].to_vec()).unwrap();
        };
        println!("app_name : {}", app_name);
        println!("---- End block ----");

        app_name
    }

    fn read_stored_file_block(block_header: &SrrBlockHeader, buffer: &[u8]) -> usize {
        if block_header.head_type != HeadType::StoredFileBlock {
            panic!("Srr block is not a Stored File Block");
        }

        if block_header.head_crc != 0x6A6A {
            panic!(
                "Stored File Block CRC is invalid : {}",
                block_header.head_crc
            );
        }

        if block_header.head_flags != 0x8000 {
            panic!("Invalid flags ! {}", block_header);
        }

        let file_size = (buffer[3] as u32) << 24
            | (buffer[2] as u32) << 16
            | (buffer[1] as u32) << 8
            | buffer[0] as u32;
        let name_size = (buffer[5] as u16) << 8 | buffer[4] as u16;
        let name = String::from_utf8(buffer[6..6 + name_size as usize].to_vec()).unwrap();
        let _stored_file_data =
            &buffer[6 + name_size as usize..6 + (name_size as u32 + file_size as u32) as usize];

        println!("file_size : {}", file_size);
        println!("name : {}", name);
        println!("---- End block ----");

        file_size as usize
    }

    fn read_rar_file_block(block_header: &SrrBlockHeader, buffer: &[u8]) {
        if block_header.head_type != HeadType::RarFileBlock {
            panic!("Rar file block is not a Header Block");
        }

        if block_header.head_crc != 0x7171 {
            panic!("Rar File Block CRC is invalid : {}", block_header.head_crc);
        }

        let file_name_size = ((buffer[1] as u16) << 8 | buffer[0] as u16) as usize;
        let file_name = String::from_utf8(buffer[2..2 + file_name_size].to_vec()).unwrap();

        println!("file name : {}", file_name);
        println!("---- End block ----");
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
        let file_size = f.read(&mut buffer).expect("buffer overflow");
        if file_size < 7 {
            panic!("File size is incoherent !");
        }

        let mut index: usize = 0;

        // Read file header
        let block_header = SrrBlockHeader::read_block_header(&buffer[0..7]);
        let application_name = SrrBlock::read_header_block(&block_header, &buffer[7..]);
        index = block_header.head_size as usize;

        // Read all blocks
        while index < file_size - 7 {
            let block_header = SrrBlockHeader::read_block_header(&buffer[index..]);

            match block_header.head_type {
                HeadType::StoredFileBlock => {
                    let add_size =
                        SrrBlock::read_stored_file_block(&block_header, &buffer[index + 7..]);
                    index += add_size;
                }
                HeadType::RarFileBlock => {
                    SrrBlock::read_rar_file_block(&block_header, &buffer[index + 7..]);
                }
                _ => {
                    break;
                }
            }

            index += block_header.head_size as usize;
        }

        SrrFile {
            application_name,
            body: vec![],
        }
    }
}
