
// Corresponds to the R:N and W:N in the memory access text
#[derive(Debug, PartialEq)]
pub enum AccessType {
    Read,
    Write,
}

// Helps us keep track of the different results when trying to access a page
// MissSimple simply means that we can push the page because we have space in physical memory
// MissReplace is when we don't have space and need to evict a page
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AccessResult {
    MissSimple,
    MissReplace(MissReplacement),
    Hit,
}
// This helps us track who was replaced, at what index, and which page replaced the old one
// Did this so we can unit test and get accurate testing results
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct MissReplacement {
    pub replaced : usize,
    pub frame_index : usize,
    pub new_page : usize,
}

impl MissReplacement {
    // Constructor
    pub fn new(replaced : usize , frame_index : usize , new_page : usize) -> MissReplacement {
        MissReplacement {
            replaced,
            frame_index,
            new_page,
        }
    }
}

// Simple data structure to represent the R:N and W:N in the text
pub struct MemoryAccess {
    pub frame_number : usize,
    pub access_type : AccessType,
}

impl MemoryAccess {
    // Constructor to create a collection by parsing the input string
    pub fn create(input_string : String) -> Vec<MemoryAccess> {
        let vals = input_string
            .split_whitespace()
            .map(|ss| ss.split(':'))
            .map(|mut ps| MemoryAccess {
                access_type : match ps.next().unwrap() {
                    "R" => AccessType::Read,
                    "W" => AccessType::Write,
                    other => panic!("Invalid access token: {}", other),
                },
                frame_number : {
                    match ps.next().unwrap().parse::<usize>() {
                        Ok(n) => n,
                        Err(e) => panic!("Invalid memory access token: {}", e)
                    }
                }});
        vals.collect()
    }
}

// Optimal and Second Chance Parameters from the CLI args
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct ORA_SCA_Params {
    pub total_frames : usize,
    pub access_string : String,
}

// WSClock Page Replacement Parameters from the CLI args
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct WSCPR_Params {
    pub total_frames : usize,
    pub access_string : String,
    pub tau : usize,
}

impl ORA_SCA_Params {
    // Constructor collecting CLI args directly, emitting errors in the process
    pub fn get() -> ORA_SCA_Params {
        let args : Vec<String> = std::env::args().collect();
        let frames = parse_number("frame count", args.get(1))
            .unwrap_or_else(|e| {
                eprintln!("Args Error: {}", e);
                std::process::exit(1);
            });
        let file = parse_file(args.get(2))
            .unwrap_or_else(|e| {
                eprintln!("Args Error: {}", e);
                std::process::exit(1);
            });
        ORA_SCA_Params {
            total_frames: frames,
            access_string: file,
        }
    }
}

impl WSCPR_Params {
    // Constructor collecting CLI args directly, emitting errors in the process
    pub fn get() -> WSCPR_Params {
        let args : Vec<String> = std::env::args().collect();
        let frames = parse_number("frame count", args.get(1))
            .unwrap_or_else(|e| {
                eprintln!("Args Error: {}", e);
                std::process::exit(1);
            });
        let tau = parse_number("tau", args.get(2))
            .unwrap_or_else(|e| {
                eprintln!("Args Error: {}", e);
                std::process::exit(1);
            });
        let file = parse_file(args.get(3))
            .unwrap_or_else(|e| {
                eprintln!("Args Error: {}", e);
                std::process::exit(1);
            });
        WSCPR_Params {
            total_frames: frames,
            access_string: file,
            tau,
        }
    }
}

pub fn get_total_faults(results : &Vec<AccessResult>) -> usize {
    results.iter()
        .filter(|r| **r != AccessResult::Hit)
        .count()
}

pub fn parse_file(filename : Option<&String>) -> Result<String, String> {
    match filename {
        None => Err(String::from("No filename provided")),
        Some(f) => std::fs::read_to_string(f).map_err(|_| format!("File {} not found", f)),
    }
}

pub fn parse_number(num_kind : &str , num_string : Option<&String>) -> Result<usize, String> {
    match num_string {
        None => Err(format!("No {} count provided", num_kind)),
        Some(a) => a.parse::<usize>().map_err(|_| format!("Invalid {} count provided", num_kind)),
    }
}
