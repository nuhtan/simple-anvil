//! # Overview
//! 
//! This crate provides a simple interface for reading the contents
//! of Minecraft region files (.rca). This crate contains no functionality
//! for writing and the reading is only to the extent of getting particular
//! blocks, getting biomes, and getting heightmaps.
//! 
//! # Example:
//! 
//! ```rust,no_run
//! use simple_anvil::region::Region
//! fn main() {
//!     let region = Region::from_file("r.0.0.mca".to_string());
//!     let chunk = region.get_chunk(2, 3).unwrap();
//!     let block = chunk.get_block(5, -12, 9);
//! 
//!     println!("{}", block.id);
//! }

/// A struct to represent a typical block in Minecraft. Really only used for gathering the name/id of a block.
pub mod block;

/// A representation of a chunk of blocks in Minecraft. 16x16x384? blocks are contained within a single chunk. This struct is used to fetch particular Blocks or to get information such as heightmaps and biomes.
pub mod chunk;

/// A representation of a region file that is used to store chunk data, functionality is limited to getting particular chunks.
pub mod region;