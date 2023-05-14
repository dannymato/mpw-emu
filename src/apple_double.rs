use binread::{BinRead, BinReaderExt, BinResult};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io;

const APPLE_DOUBLE_MAGIC: u32 = 0x00051607;

#[derive(Debug, FromPrimitive)]
#[repr(u32)]
pub enum EntryType {
    DataFork = 1,
    ResourceFork = 2,
    RealName = 3,
    Comment = 4,
    IconBW = 5,
    IconColor = 6,
    FileDates = 8,
    MacintoshFileInfo = 9,
    MSDOSFileInfo = 12,
    ShortName = 13,
    DirectoryID = 15,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct FinderInfo {
    pub type_id: u32,
    pub creator_id: u32,
    pub flags: u16,
    pub location: (i16, i16),
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct ExtendedFinderInfo {
    pub _icon_id: u16,
    pub _unused: [u8; 6],
    pub _script_flag: u8,
    pub extended_flags: u8,
    pub _comment_id: u16,
    pub home_dir_id: u32,
}

#[derive(BinRead, Debug)]
#[br(big)]
pub struct MacFileInfo {
    pub finder_info: FinderInfo,
    pub extended_info: [u8; 16],
}

#[derive(BinRead, Debug)]
#[br(big)]
pub struct Header {
    _magic: u32,
    _version_number: u32,
    _filler: [u8; 16],
    entry_count: u16,
}

#[derive(BinRead, Debug)]
#[br(big)]
struct Entry {
    entry_id: u32,
    offset: u32,
    length: u32,
}

pub struct File {
    pub header: Header,
    pub file_info: MacFileInfo,
    pub resource: Vec<u8>,
}

// For now assume that we have the appledouble and not the data file
// Can maybe add to support either file
pub fn probe(file: &[u8]) -> bool {
    u32::from_be_bytes(file[0x0..0x4].try_into().unwrap()) == APPLE_DOUBLE_MAGIC
}

pub fn unwrap(file: &[u8]) -> BinResult<File> {
    let mut cursor = io::Cursor::new(file);
    let header: Header = cursor.read_be()?;

    let mut entries: Vec<Entry> = Vec::with_capacity(header.entry_count.into());

    for _ in 0..header.entry_count {
        entries.push(cursor.read_be()?);
    }
    let mut resource: Vec<u8> = Vec::new();

    let mut file_info: Option<MacFileInfo> = None;

    for entry in entries {
        let offset = entry.offset as usize;
        let length = entry.length as usize;
        match FromPrimitive::from_u32(entry.entry_id) {
            Some(EntryType::ResourceFork) => resource = file[offset..offset + length].to_vec(),
            Some(EntryType::MacintoshFileInfo) => {
                cursor.set_position(entry.offset.into());
                file_info = Some(cursor.read_be()?);
            }
            _ => continue,
        };
    }

    let file = File {
        header,
        resource,
        file_info: file_info.unwrap(),
    };

    Ok(file)
}
