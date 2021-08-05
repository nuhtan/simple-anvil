use nbt::Blob;

use crate::chunk::Chunk;
use std::{
    array::TryFromSliceError,
    cell::Cell,
    convert::TryInto,
    fs,
    marker::{self, PhantomData},
};

#[derive(Clone)]
pub struct Region<'a> {
    data: Vec<u8>,
    _marker: marker::PhantomData<Cell<&'a ()>>,
}

impl<'a> Region<'a> {
    pub fn new(data: Vec<u8>) -> Region<'a> {
        return Region {
            data,
            _marker: PhantomData,
        };
    }

    fn header_offset(self, chunk_x: u32, chunk_z: u32) -> u32 {
        return 4 * (chunk_x % 32 + chunk_z % 32 * 32);
    }

    fn chunk_location(self, chunk_x: u32, chunk_z: u32) -> (u32, u32) {
        let off_clone = self.data.clone();
        let sec_clone = self.data.clone();
        let b_off = self.header_offset(chunk_x, chunk_z) as usize;

        let temp_range = &off_clone[b_off..b_off + 3];
        let temp: [u8; 3] = temp_range
            .try_into()
            .expect("Failed to convert slice into array.");

        let off = from_be_3_bytes(temp);
        let sectors = sec_clone[b_off as usize + 3];
        return (off, sectors as u32);
    }

    pub fn chunk_data(self, chunk_x: u32, chunk_z: u32) -> Option<Box<Blob>> {
        let temp_clone = self.data.clone();
        let comp_clone = self.data.clone();
        let comp_type_clone = self.data.clone();
        let off = self.chunk_location(chunk_x, chunk_z);
        if off == (0, 0) {
            return None;
        }
        let off: u32 = off.0 as u32 * 4096;

        let temp: Result<[u8; 4], TryFromSliceError> =
            temp_clone[off as usize..off as usize + 4].try_into();
        let length = u32::from_be_bytes(temp.unwrap());
        let compression = comp_type_clone[off as usize + 4];
        if compression == 1 {
            return None;
        }
        let compressed_data: Vec<u8> =
            comp_clone[off as usize + 5..off as usize + 5 + length as usize - 1].into();
        let data = Box::new(Blob::from_zlib_reader(&mut compressed_data.as_slice()).unwrap());
        let data = data.clone();
        return Some(data);
    }

    pub fn from_file(file: String) -> Region<'a> {
        return Region {
            data: fs::read(file).unwrap(),
            _marker: PhantomData,
        };
    }

    pub fn get_chunk(self, chunk_x: u32, chunk_z: u32) -> Chunk {
        return Chunk::from_region(self, chunk_x, chunk_z);
    }
}

fn from_be_3_bytes(bytes: [u8; 3]) -> u32 {
    let mut temp: [u8; 4] = [0; 4];
    for n in 0..bytes.len() {
        temp[n + 1] = bytes[n];
    }
    return u32::from_be_bytes(temp);
}
