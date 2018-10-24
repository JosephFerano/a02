extern crate a02;

use a02::*;

fn main() -> std::io::Result<()> {
    let params = WSCRP_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Tau: {}", params.tau);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    let v_memory : Vec<Page> = Vec::with_capacity(params.total_frames as usize);

    let results = process_page_requests(params.tau, accesses, v_memory);

    println!("Total faults: {}", &results
        .iter()
        .filter(|r| **r != AccessResult::Hit)
        .count());

    Ok(())
}


fn process_page_requests(tau : u32 , accesses : Vec<MemoryAccess> , mut v_memory : Vec<Page>) -> Vec<AccessResult> {
    let mut _pointer = 0;
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());

    for (clock, access) in accesses.iter().enumerate() {
        let contained = contains_page(access.frame_number, &v_memory);
        if contained.is_none() {
            let length = v_memory.len().clone();
            if length < v_memory.capacity() as usize {
                v_memory.push(Page {
                    number : access.frame_number,
                    timestamp : clock as u32,
                    is_dirty : (access.access_type == AccessType::Write),
                    referenced : true, });
                results.push(AccessResult::MissSimple);
            } else {
                for _ in 0..v_memory.len() {
                    let is_referenced = v_memory[0].referenced;
                    if is_referenced {
                        let mut page = v_memory.pop().unwrap();
                        page.referenced = false;
                        v_memory.push(page);
                    } else {
                        break;
                    }
                }
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        v_memory[0].number,
                        0,
                        access.frame_number)));
                v_memory[0] = Page {
                    number : access.frame_number,
                    timestamp : clock as u32,
                    is_dirty : access.access_type == AccessType::Write,
                    referenced : true };
            }
        } else {
            v_memory[contained.unwrap() as usize].referenced = true;
            if access.access_type == AccessType::Write {
                v_memory[contained.unwrap() as usize].is_dirty = true;
            }
            results.push(AccessResult::Hit);
        }
    }

    results
}

fn schedule_write_to_disk(page : Page) {
    println!("Writing to disk {:?}", page);
}

fn contains_page(page_num : u32 , collection : &Vec<Page>) -> Option<u32> {
    for (i, item) in collection.iter().enumerate() {
        if page_num == item.number {
            return Some(i as u32);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct Page {
    pub number : u32,
    pub referenced : bool,
    pub is_dirty : bool,
    pub timestamp : u32,
}

