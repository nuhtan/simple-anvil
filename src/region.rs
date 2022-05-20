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
    pub filename: String
}

impl<'a> Region<'a> {
    pub fn new(data: Vec<u8>, filename: String) -> Region<'a> {
        return Region {
            data,
            _marker: PhantomData,
            filename
        };
    }

    fn header_offset(&self, chunk_x: u32, chunk_z: u32) -> u32 {
        return 4 * (chunk_x % 32 + chunk_z % 32 * 32);
    }

    fn chunk_location(&self, chunk_x: u32, chunk_z: u32) -> (u32, u32) {
        let b_off = self.header_offset(chunk_x, chunk_z) as usize;

        let temp_range = &self.data[b_off..b_off + 3];
        let temp: [u8; 3] = temp_range
            .try_into()
            .expect("Failed to convert slice into array.");

        let off = from_be_3_bytes(temp);
        let sectors = self.data[b_off as usize + 3];
        return (off, sectors as u32);
    }

    pub fn chunk_data(&self, chunk_x: u32, chunk_z: u32) -> Option<Box<Blob>> {
        let off = self.chunk_location(chunk_x, chunk_z);
        if off == (0, 0) {
            return None;
        }
        let off: u32 = off.0 as u32 * 4096;

        let temp: Result<[u8; 4], TryFromSliceError> =
            self.data[off as usize..off as usize + 4].try_into();
        let length = u32::from_be_bytes(temp.unwrap());
        let compression = self.data[off as usize + 4];
        if compression == 1 {
            return None;
        }
        let compressed_data: Vec<u8> =
            self.data[off as usize + 5..off as usize + 5 + length as usize - 1].into();
        let data = Box::new(Blob::from_zlib_reader(&mut compressed_data.as_slice()).unwrap());
        return Some(data);
    }

    pub fn from_file(file: String) -> Region<'a> {
        return Region {
            data: fs::read(file.clone()).unwrap(),
            _marker: PhantomData,
            filename: file
        };
    }

    pub fn get_chunk(&self, chunk_x: u32, chunk_z: u32) -> Option<Chunk> {
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
