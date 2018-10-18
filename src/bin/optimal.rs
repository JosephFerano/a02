extern crate a02;

use a02::*;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    let v_memory : Vec<u32> = Vec::with_capacity(params.total_frames as usize);
    let results = process_page_requests(accesses, v_memory);

    println!("Total faults: {}", &results
        .iter()
        .filter(|r| **r != AccessResult::Hit)
        .count());

    Ok(())
}

fn process_page_requests(accesses : Vec<MemoryAccess> , mut v_memory : Vec<u32>) -> Vec<AccessResult> {
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());
    for (i, access) in accesses.iter().enumerate() {
        if !v_memory.contains(&access.frame_number) {
            let length = v_memory.len().clone();
            if length < v_memory.capacity() as usize {
                v_memory.push(access.frame_number);
//                println!("Miss! {}", access.frame_number);
                results.push(AccessResult::MissSimple);
            } else {
//                println!("Miss! {} Need to kick someone out", access.frame_number);
                let mut index = 0;
                let mut max : Option<usize> = None;
                for (ii, vm) in v_memory.iter().enumerate() {
                    for jj in (i + 1)..accesses.len() {
                        let acc = &accesses[jj];
//                        println!("Comparing {} to {}", vm, &acc.frame_number);
                        if vm == &acc.frame_number {
                            if max.is_none() || max.is_some() && max.unwrap() < jj {
                                index = ii as isize;
                                max = Some(jj);
                            }
                            break;
                        }
                    }
                    match max  {
                        Some(_) => (),
                        None => {
                            index = ii as isize;
                            break;
                        }
                    }
                }
//                println!("Removing {} from index {}, adding {}", v_memory[index as usize], index, access.frame_number);
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        v_memory[index as usize],
                        index as u32,
                        access.frame_number)));
                v_memory[index as usize] = access.frame_number;
            }
        } else {
//            println!("Hit! {}", access.frame_number);
            results.push(AccessResult::Hit);
        }
    }
    results
}
