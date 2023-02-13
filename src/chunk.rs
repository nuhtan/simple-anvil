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
    /// Contains the biome information for each section
    biome_data: Option<[[String; 64]; 24]>,
}

impl Chunk {
    
    /// Returns the chunk at an x,z coordinate within a Region.
    /// 
    /// # Arguments
    /// 
    /// * `region` - The Region from which to get the Chunk
    /// * `chunk_x` - The x coordinate within the Region of the Chunk
    /// * `chunk_z` - The z coordinate within the Region of the Chunk
    pub fn from_region(region: & Region, chunk_x: u32, chunk_z: u32) -> Option<Chunk> {
        match region.chunk_data(chunk_x, chunk_z) {
            Some(data) => {
                return Some(Chunk{ data, x: chunk_x, z: chunk_z, biome_data: None });
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
        if let Value::String(s) = self.data.get("Status").unwrap() {
            s
        } else {
            panic!("Value should be a string?")
        }
    }

    /// Returns an i64 (equivalent of Java long) of the last tick at which the chunk updated.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use simple_anvil::region::Region;
    /// let region = Region::from_file("r.0.0.mca");
    /// let chunk = region.get_chunk(0, 0).unwrap();
    /// println!("{}", chunk.get_last_update());
    /// ```
    pub fn get_last_update(&self) -> &i64 {
        if let Value::Long(l) = self.data.get("LastUpdate").unwrap() {
            l
        } else {
            panic!("Value should be a i64")
        }
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
            return Block::from_name(String::from("minecraft:air"), Some((self.x as i32 * 32 + x, y, self.z as i32 * 32 + z), ), None, String::new());
        }
        let section = section.unwrap();
        y = y.rem_euclid(16);
        let biomes = if let Some(Value::Compound(b)) = section.get("biomes") {
            b
        } else {
            panic!("Biome portion of section missing")
        };
        let pal = if let Value::List(l) = biomes.get("palette").unwrap() {
            l
        } else {
            panic!("Biome palette missing")
        };
        let data_exists = biomes.get("data");
        let biome = match data_exists {
            Some(data) => {
                let data = if let Value::LongArray(la) = data {
                    la
                } else {
                    panic!("Failed to get biome data as long array")
                };
                let dat = data[0];
                let bin = format!("{:b}", dat);
                // println!("{bin}, {}", bin.len());
                let i = bin.chars().collect::<Vec<char>>()[(((y & 0xC) << 2) | (z & 0xC) | ((x & 0xC) >> 2)) as usize].to_digit(10).unwrap();
                if let Value::String(s) = pal[i as usize].to_owned() {
                    s
                } else {
                    panic!("hah")
                }
                
            },
            None => {
                pal[0].to_string()
            },
        };
        
        let block_states = if let Some(Value::Compound(bs)) = section.get("block_states") {
            Some(bs)
        } else {
            None
        };
        if block_states == None {
            return Block::from_name(String::from("minecraft:air"), Some((self.x as i32 * 32 + x, y, self.z as i32 * 32 + z)), None, biome);
        }

        let palette = if let Value::List(p) = block_states.unwrap().get("palette").unwrap() {
            p
        } else {
            panic!("Palette should be a list")
        };

        match block_states {
            Some(bs) => {
                let bits = cmp::max(bit_length(palette.len() - 1), 4);
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
                        // let props = 
                        let props = if let Value::Compound(c) = block {
                            match c.get("Properties") {
                                Some(p_val) => {
                                    let properties = if let Value::Compound(p) = p_val {
                                        p
                                    } else {
                                        panic!("Properties should be a compound")
                                    };
                                    Some(properties.iter().map(|f| (f.0.to_owned(), if let Value::String(s) = f.1 {
                                        s.to_owned()
                                    } else {
                                        panic!("Should be a string?")
                                    })).collect::<Vec<_>>())
  
                                },
                                None => None,
                            }
                        } else {
                            panic!("block should be a compound")
                        };
                        return Block::from_palette(block, Some((self.x as i32 * 32 + x, y, self.z as i32 * 32 + z)), props, biome);
                    },
                    None => return Block::from_name(String::from("minecraft:air"), Some((self.x as i32 * 32 + x, y, self.z as i32 * 32 + z)), None, biome)
                } 
            },
            None => {
                return Block::from_name(String::from("minecraft:air"), Some((self.x as i32 * 32 + x, y, self.z as i32 * 32 + z)), None, biome);
            },
        }
        
    }

    fn fill_biome_data(mut self) {
        let mut biome_data = [[""; 64]; 24].map(|e| e.map(|se| se.to_string()));
        for n in 0..24 {
            let section = self.get_section(n).unwrap();
            let biomes = if let Some(Value::Compound(b)) = section.get("biomes") {
                b
            } else {
                panic!("Biome portion of section not found")
            };
            let biome_palette = if let Value::List(l) = biomes.get("palette").unwrap() {
                l
            } else {
                panic!("Biome palette not found")
            };
            let biome_data_exists = biomes.get("data");
            match biome_data_exists {
                Some(data) => {

                },
                None => {
                    let thing = biome_palette[0].to_string();
                    biome_data[n as usize] = [thing.as_str(); 64].map(|e| e.to_string());
                }
            }
        }
        self.biome_data = Some(biome_data);
    }
}

/// Returns the bitlength of a usize value
fn bit_length(num: usize) -> u32 {
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

fn bin_append(a: u32, b: u32, length: Option<u32>) -> u32 {
    let length = match length {
        Some(l) => l,
        None => bit_length(b as usize),
    };
    return (a << length) | b
}