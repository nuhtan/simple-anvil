#  Simple Anvil

A very barebones Anvil file parser. This is intended to be used for Minecraft related applictions. There is currently only functionality for reading the content from files. 

The sole purpose of this library is to get block data. The basic strategy is to read a region file, get a specific chunk from the region, and then get a specific block from a chunk.

Example Usage:
```rust
use simple_anvil::region::Region;

fn main() {
    let region = Region::from_file(String::from("r.0.0.mca"));
    let chunk = region.get_chunk(0, 1);
    let block = chunk.get_block(5, -20, 10);
    
    println!("Found: {}", block.name());
}
```