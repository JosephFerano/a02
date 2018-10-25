extern crate a02;

use a02::*;
use std::collections::VecDeque;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);

    let v_memory : VecDeque<Page> = VecDeque::with_capacity(params.total_frames);

    let results = process_page_requests(params.total_frames, accesses, v_memory);

    println!("Total faults: {}", get_total_faults(&results));

    Ok(())
}

fn process_page_requests(total_physical_pages : usize , accesses : Vec<MemoryAccess> , mut pages : VecDeque<Page>) -> Vec<AccessResult> {
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());

    // Iterate over all the accesses in order
    for (_i, access) in accesses.iter().enumerate() {
        let contained = contains_page(access.frame_number, &pages);
        // Does the page exist?
        if contained.is_none() {
            let length = pages.len().clone();
            // Crucially, here we check if we have space, if we do, it's a simple miss
            if length < total_physical_pages {
                pages.push_back(Page { number : access.frame_number, referenced : true });
                results.push(AccessResult::MissSimple);
            } else {
                // Iterate over the page queue...
                for _ in 0..pages.len() {
                    let is_referenced = pages[0].referenced;
                    if is_referenced {
                        // It's referenced so send to the back of the line...
                        let mut page = pages.pop_front().unwrap();
                        page.referenced = false;
                        pages.push_back(page);
                    } else {
                        // It's not referenced so just exit the loop, we're done here
                        break;
                    }
                }
                // This algorithm basically guarantees that either the unreferenced or FIFO element
                // is the one at 0, so just handle that page
                let popped = pages.pop_front().unwrap();
                pages.push_back(Page { number : access.frame_number , referenced : true });
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        popped.number,
                        0,
                        access.frame_number)));
            }
        } else {
            pages[contained.unwrap()].referenced = true;
            results.push(AccessResult::Hit);
        }
    }

    results
}

// Simple helper to check if it contains the the page number
fn contains_page(page_num : usize , collection : &VecDeque<Page>) -> Option<usize> {
    for i in 0..collection.len() {
        let item = &collection[i];
        if page_num == item.number {
            return Some(i);
        }
    }
    None
}

// Slightly more elaborate data structure than Optimal's page, helps keep track of references
#[derive(Debug, Clone)]
pub struct Page {
    pub number : usize,
    pub referenced : bool,
}

#[cfg(test)]
mod tests {
    use a02::*;
    use super::*;

    #[test]
    fn three_initial_accesses_are_all_simple_misses() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_pages = 4;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(get_total_faults(&results) , 3);
    }

    #[test]
    fn third_miss_is_miss_replace() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_pages = 2;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
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
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
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
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
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
    fn replace_the_first_one_because_first_three_are_referenced() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4"));
        let total_pages = 3;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);

        // Replaced R:1 at index 0 with R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 4);
    }

    #[test]
    fn replace_third_because_its_no_longer_referenced_after_replacing_first() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:2 R:1"));
        let total_pages = 3;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 0, pushed R:1
        let mr = MissReplacement::new(3, 0, 1);
        assert_eq!(results[5] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn replace_each_subsequent_page_because_none_are_referenced() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:5 R:6 R:4 R:5 R:6"));
        let total_pages = 3;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[6] , AccessResult::Hit);
        assert_eq!(results[7] , AccessResult::Hit);
        assert_eq!(results[8] , AccessResult::Hit);

        // Replaced R:1 at index 0 with R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:2 at index 0 with R:5
        let mr = MissReplacement::new(2, 0, 5);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 0 with R:5
        let mr = MissReplacement::new(3, 0, 6);
        assert_eq!(results[5] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 6);
    }

    #[test]
    fn replace_unreferenced_three_because_two_is_referenced() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:2 R:5"));
        let total_pages = 3;
        let v_memory : VecDeque<Page> = VecDeque::with_capacity(total_pages);
        let results = process_page_requests(total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);

        // Replaced R:1 at index 0 with R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 2 with R:5
        let mr = MissReplacement::new(3, 0, 5);
        assert_eq!(results[5] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

}
