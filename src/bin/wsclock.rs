extern crate a02;

use a02::*;

fn main() -> std::io::Result<()> {
    let params = WSCRP_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Tau: {}", params.tau);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    Ok(())
}
