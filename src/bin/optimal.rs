extern crate a02;

use a02::*;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    let mut v_memory : Vec<u32> = Vec::with_capacity(params.total_frames as usize);
    for (i, access) in accesses.iter().enumerate() {
        if !v_memory.contains(&access.frame_number) {
            let length = v_memory.len().clone();
            if length < params.total_frames as usize {
                v_memory.push(access.frame_number);
                println!("Miss! {}", access.frame_number);
            } else {
                println!("Miss! {} Need to kick someone out", access.frame_number);
                let mut index = 0;
                for (ii, vm) in v_memory.iter().enumerate() {
                    let mut found = false;
                    for jj in (i + 1)..accesses.len() {
                        let acc = &accesses[jj];
                        println!("Comparing {} to {}", vm, &acc.frame_number);
                        if !found && vm == &acc.frame_number {
                            index = ii as isize;
                            found = true;
                            break;
                        }
                    }
                    if found {
                        
                    } else {
                        index = ii as isize;
                        break;
                    }
                }
                println!("Removing {} from index {}, adding {}", v_memory[index as usize], index, access.frame_number);
                v_memory[index as usize] = access.frame_number;
            }
        } else {
            println!("Hit! {}", access.frame_number);
        }

    }
    Ok(())
}
