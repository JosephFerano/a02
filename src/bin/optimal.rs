extern crate a02;

use a02::*;

fn main() -> std::io::Result<()> {
    let params = ORA_SCA_Params::get();
    println!("Total Frames: {}", params.total_frames);
    println!("Memory accesses: {}", params.access_string);

    let accesses = MemoryAccess::create(params.access_string);
    let v_memory : Vec<usize> = Vec::with_capacity(params.total_frames);
    let results = process_page_requests(params.total_frames, accesses, v_memory);

    println!("Total faults: {}", get_total_faults(&results));

    Ok(())
}

fn process_page_requests(total_physical_pages : usize , accesses : Vec<MemoryAccess> , mut pages : Vec<usize>) -> Vec<AccessResult> {
    let mut results : Vec<AccessResult> = Vec::with_capacity(accesses.len());
    // Iterate over all the accesses in order
    for (i, access) in accesses.iter().enumerate() {
        // Does the page exist?
        if !pages.contains(&access.frame_number) {
            let length = pages.len().clone();
            // Crucially, here we check if we have space, if we do, it's a simple miss
            if length < total_physical_pages {
                pages.push(access.frame_number);
                results.push(AccessResult::MissSimple);
            } else {
                // Here we have nested loops that will go through all of the existing pages
                let mut index = 0;
                // max here keeps track of the highest index of a memory access, so that the
                // page that is accessed the latest is the one we remove
                let mut max : Option<usize> = None;
                for (ii, vm) in pages.iter().enumerate() {
                    let mut was_found = false;
                    // Only iterate from the last memory access till the end, because it doesn't
                    // make sense to start from the beginning
                    for jj in (i + 1)..accesses.len() {
                        let acc = &accesses[jj];
                        if vm == &acc.frame_number {
                            // The frame is accessed later on in the memory accesses!
                            was_found = true;
                            if max.is_none() || max.unwrap() < jj {
                                index = ii;
                                max = Some(jj);
                            }
                            break;
                        }
                    }
                    // After we initially found a page that is accessed, we later found one that
                    // wasn't
                    if !was_found && max.is_some() {
                        max = None;
                    }
                    // None means that the page is no longer accessed, so we can remove it
                    match max  {
                        Some(_) => (),
                        None => {
                            index = ii;
                            break;
                        }
                    }
                }
                // Finally! handle the replacement
                results.push(AccessResult::MissReplace(
                    MissReplacement::new(
                        pages[index],
                        index,
                        access.frame_number)));
                pages[index] = access.frame_number;
            }
        } else {
            results.push(AccessResult::Hit);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use a02::*;
    use super::*;

    #[test]
    fn three_initial_accesses_are_all_simple_misses() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_frames = 4;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(get_total_faults(&results) , 3);
    }

    #[test]
    fn third_miss_is_miss_replace() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 W:3"));
        let total_frames = 2;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
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
        let total_frames = 2;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        for i in 1..results.len() {
            assert_eq!(results[i] , AccessResult::Hit);
        }
        assert_eq!(get_total_faults(&results) , 1);
    }

    #[test]
    fn alternating_hits_and_misses() {
        let accesses = MemoryAccess::create(String::from("R:1 W:1 W:2 R:1 R:2 W:3 W:4"));
        let total_frames = 4;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
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
    fn replace_the_last_one_to_be_accessed() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:1 R:2 R:3"));
        let total_frames = 3;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);
        assert_eq!(results[5] , AccessResult::Hit);

        // Replaced R:3 at index 2 with R:4
        let mr = MissReplacement::new(3, 2, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:1 at index 0 with R:3
        let mr = MissReplacement::new(1, 0, 3);
        assert_eq!(results[6] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn replace_second_because_its_no_longer_accessed() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:1 R:4"));
        let total_frames = 3;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[4] , AccessResult::Hit);
        assert_eq!(results[5] , AccessResult::Hit);

        // Replaced R:2 at index 1 with R:4
        let mr = MissReplacement::new(2, 1, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 4);
    }

    #[test]
    fn replace_first_and_third_because_they_are_no_longer_accessed() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:5 R:2 R:4"));
        let total_frames = 3;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);

        // Replaced R:1 at index 0 with R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 2 with R:5
        let mr = MissReplacement::new(3, 2, 5);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 5);
    }

    #[test]
    fn replace_first_twice_because_of_miss() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4"));
        let total_frames = 2;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);

        // Replaced R:1 at index 0 with R:3
        let mr = MissReplacement::new(1, 0, 3);
        assert_eq!(results[2] , AccessResult::MissReplace(mr));

        // Replaced R:3 at index 0 with R:4
        let mr = MissReplacement::new(3, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 4);
    }

    #[test]
    fn replace_first_thrice() {
        let accesses = MemoryAccess::create(String::from("R:1 R:2 R:3 R:4 R:5 R:3 R:2 R:1"));
        let total_frames = 3;
        let v_memory : Vec<usize> = Vec::with_capacity(total_frames);
        let results = process_page_requests(total_frames, accesses, v_memory);
        assert_eq!(results[0] , AccessResult::MissSimple);
        assert_eq!(results[1] , AccessResult::MissSimple);
        assert_eq!(results[2] , AccessResult::MissSimple);
        assert_eq!(results[5] , AccessResult::Hit);
        assert_eq!(results[6] , AccessResult::Hit);

        // Replaced R:3 at index 2 with R:4
        let mr = MissReplacement::new(1, 0, 4);
        assert_eq!(results[3] , AccessResult::MissReplace(mr));

        // Replaced R:4 at index 0 with R:5
        let mr = MissReplacement::new(4, 0, 5);
        assert_eq!(results[4] , AccessResult::MissReplace(mr));

        // Replaced R:5 at index 0 with R:1
        let mr = MissReplacement::new(5, 0, 1);
        assert_eq!(results[7] , AccessResult::MissReplace(mr));

        assert_eq!(get_total_faults(&results) , 6);
    }

}