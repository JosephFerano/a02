extern crate a02;

use a02::*;
use std::collections::VecDeque;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);

    let v_memory : VecDeque<Page> = VecDeque::with_capacity(params.total_frames as usize);

    let results = process_page_requests(accesses, v_memory);

    println!("Size {}", results.len());
    println!("Total faults: {}", &results
        .iter()
        .filter(|r| **r != AccessResult::Hit)
        .count());

    Ok(())
}

fn process_page_requests(accesses : Vec<MemoryAccess> , mut v_memory : VecDeque<Page>) -> Vec<AccessResult> {
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());

    for (_i, access) in accesses.iter().enumerate() {
        let contained = contains_page(access.frame_number, &v_memory);
        if contained.is_none() {
            let length = v_memory.len().clone();
            if length < v_memory.capacity() as usize {
                v_memory.push_back(Page { number : access.frame_number, referenced : true });
                results.push(AccessResult::MissSimple);
            } else {
                for _ in 0..v_memory.len() {
                    let is_referenced = v_memory[0].referenced;
                    if is_referenced {
                        let mut page = v_memory.pop_front().unwrap();
                        page.referenced = false;
                        v_memory.push_back(page);
                    } else {
                        break;
                    }
                }
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        v_memory[0].number,
                        0,
                        access.frame_number)));
                v_memory[0] = Page { number : access.frame_number , referenced : true };
            }
        } else {
            v_memory[contained.unwrap() as usize].referenced = true;
            results.push(AccessResult::Hit);
        }
    }

    results
}

fn contains_page(page_num : u32 , collection : &VecDeque<Page>) -> Option<u32> {
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
}

