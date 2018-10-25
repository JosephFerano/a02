extern crate a02;

use a02::*;
use std::cmp::Ordering;

fn main() -> std::io::Result<()> {
    let params = WSCRP_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Tau: {}", params.tau);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    let v_memory : Vec<Page> = Vec::with_capacity(params.total_frames);

    let results = process_page_requests(params.tau, params.total_frames, accesses, v_memory);

    println!("Total faults: {}", get_total_faults(&results));

    Ok(())
}

fn process_page_requests(tau : usize , total_physical_pages : usize , accesses : Vec<MemoryAccess> , mut v_memory : Vec<Page>)-> Vec<AccessResult> {
    let mut pointer = 0;
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());

    for (clock, access) in accesses.iter().enumerate() {
        let contained = contains_page(access.frame_number, &v_memory);
        if contained.is_none() {
            let length = v_memory.len().clone();
            if length < total_physical_pages {
                v_memory.push(Page {
                    number : access.frame_number,
                    timestamp : clock,
                    is_dirty : false,
                    referenced : true, });
                results.push(AccessResult::MissSimple);
//                println!("MissSimple {}", access.frame_number);
            } else {
                let mut index = 0;
                let start_pointer = pointer % length;
                let mut iteration = 0;
                // Iterate once to
                loop {
                    let page = &mut v_memory[pointer % length];
                    println!("Checking {:?} Iter {} Age {}", page, iteration, clock - page.timestamp);
                    if page.referenced {
                        page.referenced = false;
                    } else {
                        let age = clock - page.timestamp;
                        if age > tau {
                            if !page.is_dirty {
                                index = pointer % length;
                                println!("OldAndClean {}", page.number);
                                break;
                            }
                        } else if iteration > 0 {
                            if !page.is_dirty {
                                index = pointer % length;
                                println!("OldAndClean {}", page.number);
                                break;
                            }
                        }
                        if page.is_dirty {
                            println!("OldAndDirty {}", page.number);
                            schedule_write_to_disk(page.clone());
                            page.is_dirty = false;
                        }
                    }
                    pointer += 1;
                    if start_pointer == pointer % length {
                        iteration += 1;
                    }
                }
                let i = index;
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        v_memory[i].number,
                        i,
                        access.frame_number)));
                v_memory[i] = Page {
                    number : access.frame_number,
                    timestamp : clock,
                    is_dirty : false,
                    referenced : true };
            }
            println!("-----------------")
        } else {
//            println!("Hit {:?}", v_memory[contained.unwrap()].number);
            v_memory[contained.unwrap()].referenced = true;
            v_memory[contained.unwrap()].timestamp = clock;
            if access.access_type == AccessType::Write {
                v_memory[contained.unwrap()].is_dirty = true;
            }
            results.push(AccessResult::Hit);
        }
    }

    results
}

fn schedule_write_to_disk(page : Page) {
//    println!("Scheduling write to disk {:?}", page);
}

fn contains_page(page_num : usize , collection : &Vec<Page>) -> Option<usize> {
    for (i, item) in collection.iter().enumerate() {
        if page_num == item.number {
            return Some(i);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct Page {
    pub number : usize,
    pub referenced : bool,
    pub is_dirty : bool,
    pub timestamp : usize,
}

#[cfg(test)]
mod tests {
    use a02::*;
    use super::*;

    #[test]
    fn three_initial_accesses_are_all_simple_misses() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_pages = 4;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(get_total_faults(&results) , 3);
    }

    #[test]
    fn third_miss_is_miss_replace() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_pages = 2;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);

        // Replaced R:1 at index 0 with R:3
        let mr = MissReplacement::new(1, 0, 3);
        assert_eq!(results[2] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 3);
    }

    #[test]
    fn all_are_subsequent_hits_after_first_miss() {
        let accesses = MemoryAccess::create(String::from("R:1 R:1 W:1 R:1 W:1"));
        let total_pages = 2;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        for i in 1..results.len() {
            assert_eq!(results[i] , AccessResult::Hit);
        }
        assert_eq!(get_total_faults(&results) , 1);
    }

    #[test]
    fn alternating_hits_and_misses() {
        let accesses = MemoryAccess::create(String::from("R:1 W:1 W:2 R:1 R:2 W:3 W:4"));
        let total_pages = 4;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::Hit);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[3] , AccessResult::Hit);
        assert_eq!(results[4] , AccessResult::Hit);
        assert_eq!(results[5] , AccessResult::MissSimple);
        assert_eq!(results[6] , AccessResult::MissSimple);
        assert_eq!(results.len() , 7);
        assert_eq!(get_total_faults(&results) , 4);
    }

    #[test]
    fn evicts_first_then_four_since_two_and_three_are_later_referenced() {
        let accesses = MemoryAccess::create(String::from("R:1 W:2 W:3 R:4 R:2 W:3 W:5"));
        let total_pages = 3;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);
        assert_eq!(results[5] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(4, 0, 5);
        assert_eq!(results[6] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn evicts_one_and_four_because_age_is_old_enough() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 W:4 W:2 R:5"));
        let total_pages = 3;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:4 at index 0, pushed R:5
        let mr = MissReplacement::new(4, 0, 5);
        assert_eq!(results[5] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn skips_dirty_page_two_for_clean_three_when_age_is_old() {
        let accesses = MemoryAccess::create(String::from("R:1 W:2 R:3 W:2 W:4 R:4 R:4 R:5"));
        let total_pages = 3;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(3, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[3] , AccessResult::Hit);
        assert_eq!(results[5] , AccessResult::Hit);
        assert_eq!(results[6] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 2, pushed R:5
        let mr = MissReplacement::new(3, 2, 5);
        assert_eq!(results[7] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

}
