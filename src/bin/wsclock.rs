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
            } else {
                let length = v_memory.len();
                let mut candidate = Candidate::new(pointer % length, PageStatus::None);
                for _ in 0..length {
                    let page = &mut v_memory[pointer % length];
                    if page.referenced {
                        page.referenced = false;
                        pointer += 1;
                    } else {
                        let age = clock - page.timestamp;
                        if age > tau {
                            // Check to see if there's a clean page further down
                            if page.is_dirty {
                                let new_candidate = Candidate::new(pointer % length, PageStatus::OldAndDirty);
                                if candidate < new_candidate { candidate = new_candidate; }
                                println!("OldAndDirty {}", page.number);
                                schedule_write_to_disk(page.clone());
                            } else {
                                // We can end the for loop, we found an old page that's clean
                                let new_candidate = Candidate::new(pointer % length, PageStatus::OldAndClean);
                                if candidate < new_candidate { candidate = new_candidate; }
                                println!("OldAndClean {}", page.number);
                                break;
                            }
                        } else {
                            if page.is_dirty {
                                let new_candidate = Candidate::new(pointer % length, PageStatus::Dirty);
                                if candidate < new_candidate { candidate = new_candidate; }
                                println!("Dirty {}", page.number);
                                schedule_write_to_disk(page.clone());
                            } else {
                                let new_candidate = Candidate::new(pointer % length, PageStatus::Clean);
                                if candidate < new_candidate { candidate = new_candidate; }
                                println!("Clean {}", page.number);
                            }
                        }
                        pointer += 1;
                    }
                }
//                println!("{:?}", candidate.status);
                let index = candidate.index;
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        v_memory[index].number,
                        index,
                        access.frame_number)));
                v_memory[index] = Page {
                    number : access.frame_number,
                    timestamp : clock,
                    is_dirty : false,
                    referenced : true };
            }
        } else {
            println!("Hit {:?}", v_memory[contained.unwrap()].number);
            v_memory[contained.unwrap()].referenced = true;
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

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
struct Candidate {
    index : usize,
    status : PageStatus,
}

impl Candidate {
    pub fn new(index : usize , status : PageStatus) -> Candidate { Candidate { index , status } }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
enum PageStatus {
    None,
    Dirty,
    Clean,
    OldAndDirty,
    OldAndClean,
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.status.cmp(&other.status)
    }
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
    fn candidate_comparison_works() {
        let d_candidate = Candidate { index: 0, status: PageStatus::Dirty };
        let c_candidate = Candidate { index: 0, status: PageStatus::Clean };
        let oad_candidate = Candidate { index: 0, status: PageStatus::OldAndDirty };
        let oac_candidate = Candidate { index: 0, status: PageStatus::OldAndClean };
        assert_eq!(d_candidate < c_candidate , true);
        assert_eq!(d_candidate < oad_candidate , true);
        assert_eq!(d_candidate < oac_candidate , true);
        assert_eq!(oac_candidate > oad_candidate , true);
    }

    #[test]
    fn evicts_three_because_two_is_dirty_after_evicting_one() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:2 R:3 W:4 R:5"));
        let total_pages = 3;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::Hit);
        assert_eq!(results[3] , AccessResult::MissSimple);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 2, pushed R:5
        let mr = MissReplacement::new(3, 2, 5);
        assert_eq!(results[5] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn evicts_four_because_two_and_three_are_referenced() {
        let accesses = MemoryAccess::create(String::from("R:1 W:2 R:3 W:4 R:5 R:2 R:3 R:6"));
        let total_pages = 4;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[3] , AccessResult::MissSimple);
        assert_eq!(results[5] , AccessResult::Hit);
        assert_eq!(results[6] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 5);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(4, 3, 6);
        assert_eq!(results[7] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 6);
    }

    #[test]
    fn evicts_because_tau_is_older() {
        let accesses = MemoryAccess::create(String::from("R:1 W:2 R:3 W:4 R:5 R:2 R:3 R:6"));
        let total_pages = 4;
        let v_memory : Vec<Page> = Vec::with_capacity(total_pages);
        let results = process_page_requests(5, total_pages, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[3] , AccessResult::MissSimple);
        assert_eq!(results[5] , AccessResult::Hit);
        assert_eq!(results[6] , AccessResult::Hit);

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(1, 0, 5);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:1 at index 0, pushed R:4
        let mr = MissReplacement::new(4, 3, 6);
        assert_eq!(results[7] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 6);
    }

}
