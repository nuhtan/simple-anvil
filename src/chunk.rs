use nbt::{Blob, Value};

use crate::{block::Block, region::Region};

use std::{cmp, collections::HashMap};

#[derive(Clone)]
pub struct Chunk {
    pub data: Box<Blob>,
}

impl Chunk {
    pub fn new(nbt_data: Box<Blob>) -> Chunk {
        let level_data = nbt_data;
        Chunk { data: level_data }
    }

    pub fn from_region(region: &Region, chunk_x: u32, chunk_z: u32) -> Option<Chunk> {
        match region.chunk_data(chunk_x, chunk_z) {
            Some(data) => {
                let chunk = Chunk::new(data);
                return Some(chunk);
            }
            None => None,
        }
    }

    fn get_section(&self, y: i8) -> Option<HashMap<String, Value>> {
        if y < -4 || y > 19 {
            panic!("Y value out of range")
        }
        let sections = if let Value::List(s) = self.data.get("sections").unwrap() {
            s
        } else {
            panic!("Value should be a list?")
        };

        for section in sections {
            let section = if let Value::Compound(s) = section {
                s
            } else {
                panic!("should be a compound")
            };
            let section_y = if let Value::Byte(sec_y) = section.get("Y").unwrap() {
                sec_y
            } else {
                panic!("Failed to get y")
            };
            if *section_y == y {
                let cloned = section.clone();
                return Some(cloned);
            }
        }
        None
    }

    pub fn get_biome(&self, y: i32) -> String {
        let sections = if let Value::List(s) = self.data.get("sections").unwrap() {
            s
        } else {
            panic!("Value should be a list?")
        };
        for section in sections {
            let section = if let Value::Compound(s) = section {
                s
            } else {
                panic!("Should be a compound?")
            };
            let current_y = if let Value::Byte(val) = section.get("Y").unwrap() {
                val
            } else {
                panic!("invalid height found")
            };
            if current_y == &(((y + 64) / 16 - 4) as i8) {
                let biomes = if let Value::Compound(c) = section.get("biomes").unwrap() {
                    c
                } else {
                    panic!("biomes not found")
                };
                let pallete = if let Value::List(l) = biomes.get("palette").unwrap() {
                    l
                } else {
                    panic!("pallete not found")
                };
                let biome = if let Value::String(s) = &pallete[0] {
                    s
                } else {
                    panic!("failed to get string")
                };
                return biome.to_string();
            }
            
        };
        return String::from("minecraft:ocean")
    }

    pub fn get_block(&self, x: i32, mut y: i32, z: i32) -> Block {
        let section = self.get_section(((y + 64) / 16 - 4) as i8);
        if section == None {
            return Block::from_name(String::from("minecraft:air"));
        }
        let section = section.unwrap();
        y = y.rem_euclid(16);
        let block_states = if let Some(Value::Compound(bs)) = section.get("block_states") {
            Some(bs)
        } else {
            None
        };
        if block_states == None {
            return Block::from_name(String::from("minecraft:air"));
        }

        let palette = if let Value::List(p) = block_states.unwrap().get("palette").unwrap() {
            p
        } else {
            panic!("Palette should be a list")
        };
        match block_states {
            Some(bs) => {
                let bits = cmp::max(self.bit_length(palette.len() - 1), 4);
                let index = y * 16 * 16 + z * 16 + x;
                match bs.get("data") {
                    Some(data) => {
                        let states = if let Value::LongArray(la) = data {
                            la
                        } else {
                            panic!("something here")
                        };
                        let state = index as usize / (64 / bits as usize);
                        let data = states[state];
                        let mut d = 0;
                        let mut modified = false;
                        if data < 0 {
                            d = data as u64;
                            modified = true;
                        }
                        let shifted_data = (if modified { d as usize } else { data as usize }) >> (index as usize % (64 / bits as usize) * bits as usize);
                        let palette_id = shifted_data & (2u32.pow(bits) - 1) as usize;
                        let block = &palette[palette_id];
                        return Block::from_palette(block);
                    },
                    None => return Block::from_name(String::from("minecraft:air")),
                } 
            },
            None => {
                return Block::from_name(String::from("minecraft:air"));
            },
        }
        
    }

    fn bit_length(&self, num: usize) -> u32 {
        // The number of bits that the number consists of, this is an integer and we don't care about signs or leading 0's
        // 0001 and 1 have the same return value
        // I think the lowest number that could come in is -1?
        // usize is always returned from the len function so I think that it will only be usize?
        if num == 0 {
            return 0;
        }
        // Convert the number to a string version of the binary representation
        // Get the number of leading 0's
        let _leading = num.leading_zeros();
        // Place the number into binary
        let s_num = format!("{:b}", num);
        // Remove leading 0's
        // let s = &s_num[leading as usize..];
        // Return the length
        // Leading zeros appear to be removed when changed to bits
        return s_num.len() as u32;
    }
}
