use nbt::Blob;

use crate::{chunk::Chunk, block::Block};

use std::{
    array::TryFromSliceError,
    cell::Cell,
    convert::TryInto,
    fs,
    marker::{self, PhantomData},
    path::Path,
};

/// Low level storage of region file contents.
#[derive(Clone)]
pub struct Region<'a> {
    /// Vector containing all of the data in bytes.
    data: Vec<u8>,
    /// I don't remember what this was for.
    _marker: marker::PhantomData<Cell<&'a ()>>,
    /// The name of the file that the region was derived from.
    pub filename: String,
}

impl<'a> Region<'a> {
    /// Returns the header size and returns an offset for a particular chunk.
    /// 
    /// # Arguments
    /// 
    /// * `chunk_x` - The x coordinate of the particular chunk
    /// * `chunk_z` - The z coordinate of the particular chunk
    fn header_offset(&self, chunk_x: u32, chunk_z: u32) -> u32 {
        return 4 * (chunk_x % 32 + chunk_z % 32 * 32);
    }

    /// Returns the location where a particular chunk is found.
    ///
    /// # Arguments
    /// 
    /// * `chunk_x` - The x coordinate of the particular chunk
    /// * `chunk_z` - The z coordinate of the particular chunk
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

    /// Returns a Blob of all the data for a particular chunk. 
    /// 
    /// # Arguments
    /// 
    /// * `chunk_x` - The x coordinate of the particular chunk
    /// * `chunk_z` - The z coordinate of the particular chunk
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

    /// Returns a region using a region(.mca) file
    /// 
    /// # Arguments
    /// 
    /// * `file` - The file name and relative path of the region file.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// 
    /// let region = Region::from_file("r.0.0.mca".into());
    /// ```
    pub fn from_file(file: String) -> Region<'a> {
        let f = Path::new(&file);
        return Region {
            data: fs::read(file.clone()).unwrap(),
            _marker: PhantomData,
            filename: f.file_name().unwrap().to_str().unwrap().to_string(),
        };
    }

    /// Returns a Chunk contained within the Region. A region file contains 32x32 chunks.
    /// 
    /// # Arguments
    /// 
    /// * `chunk_x` - The x coordinate of the particular chunk
    /// * `chunk_z` - The z coordinate of the particular chunk
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// 
    /// let region = Region::from_file("r.0.0.mca".into());
    /// let chunk = region.get_chunk(11, 2).unwrap();
    /// ```
    pub fn get_chunk(&self, chunk_x: u32, chunk_z: u32) -> Option<Chunk> {
        return Chunk::from_region(self, chunk_x, chunk_z);
    }


    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        let chunk = self.get_chunk((x / 32) as u32, (z / 32) as u32);
        return match chunk {
            Some(c) => {
                Some(c.get_block(x % 32, y, z % 32))
            },
            None => None,
        }
    }
}

/// Returns an unsigned int from three bytes. This might not be needed anymore.
/// 
/// # Arguments
/// 
/// * `bytes` - The bytes to be converted into u32
fn from_be_3_bytes(bytes: [u8; 3]) -> u32 {
    let mut temp: [u8; 4] = [0; 4];
    for n in 0..bytes.len() {
        temp[n + 1] = bytes[n];
    }
    return u32::from_be_bytes(temp);
}
