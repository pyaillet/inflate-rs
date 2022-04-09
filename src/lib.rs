#![allow(dead_code)]

use std::io::Read;

use log::*;

use byteorder::{LittleEndian, ReadBytesExt};

use bitmask_enum::bitmask;
use num_enum::FromPrimitive;

const MAGIC1: u8 = 0x1f;
const MAGIC2: u8 = 0x8b;

#[derive(Debug, Clone, Copy)]
enum FormatError {
    InvalidMagic1(u8),
    InvalidMagic2(u8),
}

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Format(FormatError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug)]
struct Member {
    magic1: u8,
    magic2: u8,
    compression_method: CompressionMethod,
    flags: Flags,
    modification_time: u32,
    extra_flags: u8,
    operating_system: OperatingSystem,
    extra_fields: Option<Extra>,
    original_file_name: Option<String>,
    comment: Option<String>,
    crc16: Option<u16>,
    data: Vec<u8>,
    crc32: u32,
    size: u32,
}

#[derive(Clone, Debug)]
struct Extra {
    subid1: u8,
    subid2: u8,
    length: u16,
    data: Vec<u8>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
enum CompressionMethod {
    Reserved0 = 0x00,
    Reserved1 = 0x01,
    Reserved2 = 0x02,
    Reserved3 = 0x03,
    Reserved4 = 0x04,
    Reserved5 = 0x05,
    Reserved6 = 0x06,
    Reserved7 = 0x07,
    Deflate = 0x08,
    #[default]
    Unknown = 0xff,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
enum OperatingSystem {
    Fat = 0x00,
    Amiga = 0x01,
    Vms = 0x02,
    Unix = 0x03,
    VmCms = 0x04,
    Atari = 0x05,
    Hpfs = 0x06,
    Macintosh = 0x07,
    ZSystem = 0x08,
    CPM = 0x09,
    Top20 = 0x0a,
    Ntfs = 0x0b,
    QDos = 0x0c,
    AcornRiscOS = 0x0d,
    #[default]
    Unknown = 0xff,
}

#[bitmask(u8)]
#[derive(Clone, Copy, Debug)]
enum Flags {
    Text = Self(0b0000_0001),
    HCrc = Self(0b0000_0010),
    Extra = Self(0b0000_0100),
    Name = Self(0b0000_1000),
    Comment = Self(0b0001_0000),
}

impl Member {
    fn from_reader(mut r: impl Read) -> Result<Self, Error> {
        let magic1 = r.read_u8()?;
        if magic1 != MAGIC1 {
            return Err(Error::Format(FormatError::InvalidMagic1(magic1)));
        }
        let magic2 = r.read_u8()?;
        if magic2 != MAGIC2 {
            return Err(Error::Format(FormatError::InvalidMagic2(magic2)));
        }
        let compression_method: CompressionMethod = r.read_u8()?.into();
        let flags: Flags = r.read_u8()?.into();
        let modification_time = r.read_u32::<LittleEndian>()?;
        let extra_flags = r.read_u8()?;
        let operating_system = r.read_u8()?.into();
        let extra_fields = if flags.contains(Flags::Extra) {
            let subid1 = r.read_u8()?;
            let subid2 = r.read_u8()?;
            let length = r.read_u16::<LittleEndian>()?;
            let data = read_to_vec(&mut r, length)?;
            Some(Extra {
                subid1,
                subid2,
                length,
                data,
            })
        } else {
            None
        };
        let original_file_name = if flags.contains(Flags::Name) {
            let mut buf: Vec<u8> = Vec::new();
            loop {
                match r.read_u8()? {
                    0 => {
                        break;
                    }
                    x => {
                        buf.push(x);
                    }
                }
            }
            Some(String::from_utf8_lossy(&buf).to_string())
        } else {
            None
        };
        let comment = if flags.contains(Flags::Comment) {
            let mut buf: Vec<u8> = Vec::new();
            loop {
                match r.read_u8()? {
                    0 => {
                        break;
                    }
                    x => {
                        buf.push(x);
                    }
                }
            }
            Some(String::from_utf8_lossy(&buf).to_string())
        } else {
            None
        };
        let crc16 = if flags.contains(Flags::HCrc) {
            Some(r.read_u16::<LittleEndian>()?)
        } else {
            None
        };
        let data = Vec::new();
        let crc32 = 0;
        let size = 0;

        Ok(Member {
            magic1,
            magic2,
            compression_method,
            flags,
            modification_time,
            extra_flags,
            operating_system,
            extra_fields,
            original_file_name,
            comment,
            crc16,
            data,
            crc32,
            size,
        })
    }
}

fn read_to_vec<T>(r: &mut impl Read, length: T) -> Result<Vec<u8>, std::io::Error>
where
    T: Into<usize>,
{
    let length: usize = length.into();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        vec.push(r.read_u8()?);
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs::File, io::BufReader};

    use crate::Member;

    // const FILE: &str = "./tests/test.gz";
    const FILE: &str = "../_Recette.doc.extracted/1E6200.tar.gz";

    #[test]
    fn test_member_from_reader() {
        let f = File::open(FILE);
        assert!(f.is_ok());
        let f = f.unwrap();
        let reader = BufReader::new(f);
        let member = Member::from_reader(reader);
        assert!(member.is_ok());
        let member = member.unwrap();

        println!("{:?}", &member);
    }
}
