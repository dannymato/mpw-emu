use binread::{BinRead, BinReaderExt, BinResult};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{io, fs};
use std::path::Path;

const APPLE_DOUBLE_MAGIC: u32 = 0x00051607;

#[derive(Debug, FromPrimitive)]
#[allow(dead_code)]
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

fn is_apple_double(file: &[u8]) -> bool {
    if file.len() < 4 {
        return false;
    }

    let res = file[0x0..0x4].try_into();

    res.is_ok() && u32::from_be_bytes(res.unwrap()) == APPLE_DOUBLE_MAGIC
}

pub fn probe<P: AsRef<Path>>(file: &[u8], path: P) -> Option<(Vec<u8>, Vec<u8>)> {
    if is_apple_double(&file) {
        return Some((file.to_vec(), find_data_file(path)?));
    }

    Some((find_resource_file(path)?, file.to_vec()))
}

fn find_resource_file<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    let path: &Path = path.as_ref();
    if let Some(parent) = path.parent() {
        // Trying original.rsrc
        let new_path = parent.join(path.with_extension("rsrc"));
        if new_path.exists() {
            return read_resource(new_path);
        }

        // Trying ._original
        if let Some(file_name) = path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                let new_path = parent.join(format!("._{file_name}"));
                if new_path.exists() {
                    return read_resource(new_path);
                }
            }
        }
    }

    None
}

fn read_resource<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    if let Ok(file_contents) = fs::read(path) {
        if is_apple_double(&file_contents) {
            return Some(file_contents)
        }
    }
    None
}

fn find_data_file<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    let path: &Path = path.as_ref();
    if let Some(parent) = path.parent() {
        if let Some(stem) = path.file_stem() {
            let new_path = parent.join(stem);

            if !new_path.exists() {
                return None;
            }

            if let Ok(data_contents) = fs::read(new_path) {
                return Some(data_contents);
            }
        }
    }

    None
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
