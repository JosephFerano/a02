extern crate a02;

use a02::*;
use std::collections::VecDeque;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);

    let mut v_memory : VecDeque<u32> = VecDeque::with_capacity(params.total_frames as usize);

    Ok(())
}
