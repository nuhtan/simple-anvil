use nbt::{Blob, Value};

use crate::{block::Block, region::Region};

use std::{cmp, collections::HashMap};

/// A simple representation of a Minecraft Chunk
#[derive(Clone)]
pub struct Chunk {
    /// All of the chunk data
    pub data: Box<Blob>,
    /// The region x
    pub x: u32,
    /// The region z
    pub z: u32,
}

impl Chunk {
    
    /// Returns the chunk at an x,z coordinate within a Region.
    /// 
    /// # Arguments
    /// 
    /// * `region` - The Region from which to get the Chunk
    /// * `chunk_x` - The x coordinate within the Region of the Chunk
    /// * `chunk_z` - The z coordinate within the Region of the Chunk
    pub fn from_region(region: &Region, chunk_x: u32, chunk_z: u32) -> Option<Chunk> {
        match region.chunk_data(chunk_x, chunk_z) {
            Some(data) => {
                return Some(Chunk{ data, x: chunk_x, z: chunk_z });
            }
            None => None,
        }
    }

    /// Returns a string representing the current generation state of the Chunk. 'full' is completely generated.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// if chunk.get_status() == "full" {
    ///     println!("Fully Generated!");
    /// }
    /// ```
    pub fn get_status(&self) -> &String {
        let status = if let Value::String(s) = self.data.get("Status").unwrap() {
            s
        } else {
            panic!("Value should be a string?")
        };
        status
    }

    /// Returns a heightmap of the Chunk. If the Chunk is not fully generated then a None is returned.
    /// 
    /// # Arguments
    /// 
    /// * `ignore_water` - Determines which heightmap to return, if true then a heightmap that does not take into account the water is returned (OCEAN_FLOOR), if false then the water is accounted for (WORLD_SURFACE).
    /// 
    ///  # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// let heightmap = chunk.get_heightmap(false);
    /// ```
    pub fn get_heightmap(&self, ignore_water: bool) -> Option<Vec<i32>> {
        if self.get_status() == "full" {
            let height_maps = if let Value::Compound(hm) = self.data.get("Heightmaps").unwrap() {
                hm
            } else {
                panic!()
            };

            let map = if ignore_water {
                "OCEAN_FLOOR"
            } else {
                "WORLD_SURFACE"
            };

            let surface = if let Value::LongArray(la) = height_maps.get(map).unwrap() {
                la
            } else {
                panic!("no ocean?")
            };

            let surface_binary: Vec<String> = surface.iter().map(|n| format!("{:b}", n)).map(|n| "0".repeat(63 - n.len()) + &n).collect();
            let mut all = Vec::new();
            // let mut hmm = Vec::new();

            for num in surface_binary {
                let num_chars = num.chars().collect::<Vec<_>>();
                let mut sub_nums = num_chars.chunks(9).collect::<Vec<&[char]>>();
                sub_nums.reverse();
                for num in sub_nums {
                    let test = num.iter().collect::<String>();
                    if test != "000000000" {
                        all.push(test.clone());
                    }
                }
            }

            let mut heights = Vec::new();

            for num in all {
                let n = usize::from_str_radix(num.as_str(), 2).unwrap();
                heights.push(n as i32 - 64 - 1);
            }

            return Some(heights);
        } else {
            None
        }
    }

    /// Returns a vertical section of a Chunk
    /// 
    /// # Arguments
    /// 
    /// * `y` - The y index of the section.
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

    /// Returns the String representation of the biome for a Chunk. Chunks can have different biomes at different vertical sections so use a heightmap to determine the top section if you only want the surface biome.
    /// 
    /// # Arguments
    /// 
    /// * `y` - The y section of the chunk to get the biome of.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// let heightmap = chunk.get_heightmap(false);
    /// let y = if let Some(heights) = heightmap {
    ///     heights.get(0).unwrap()
    /// } else {
    ///     panic!("Chunk not fully generated");
    /// }
    /// let section_y = ((y + 64) / 16 - 4) as i8
    /// let biome = chunk.get_biome(section_y);
    /// ```
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// let biome = chunk.get_biome(-3);
    /// ```
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

    /// Returns the block at a particular x, y, z coordinate within a chunk. x and z should be the coordinates within the Chunk (0-15).
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// let block = chunk.get_block(5, -12, 11);
    /// println!("{}", block.id);
    /// ```
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

    /// Returns the bitlength of a usize value
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
