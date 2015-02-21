#![feature(old_io,old_path,core,os,plugin)]

#![plugin(regex_macros)]
#[no_link] extern crate regex_macros;
extern crate regex;

extern crate getopts;

use regex::Regex;
use std::os;
use std::old_io::{File, fs};
use std::old_io::fs::PathExtensions;
use std::collections::HashSet;
use getopts::Options;

struct Matcher {
    exp: Regex,
}

impl Matcher {
    fn new() -> Matcher {
        Matcher { exp: regex!(r"^shutterstock_(\d+)") }
    }

    fn image_number(&self, test: &str) -> Option<String> {
        match self.exp.captures(test) {
            Some(cap) => { match cap.at(1) {
                Some(s) => return Some(s.to_string()),
                None => return None,
            }},
            None => return None,
        }
    }
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "input-dir", "the folder that files will be deleted from (required)", "DIRECTORY");
    opts.optopt("m", "master-dir", "the folder that will be used as a master list (required)", "DIRECTORY");
    opts.optopt("o", "output-file", "the file to output the log to", "OUTPUT");
    opts.optflag("d", "delete", "automatically delete the duplicate files");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("q", "quiet", "don't display duplicates as they are found");

    let matches = match opts.parse(args.tail()) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let input_directory = match matches.opt_str("i") {
        Some(d) => d,
        None => {
            print_usage(&program, opts);
            return;
        },
    };

    let master_directory = match matches.opt_str("m") {
        Some(d) => d,
        None => {
            print_usage(&program, opts);
            return;
        },
    };

    let quiet = matches.opt_present("q");

    let input_files = fs::walk_dir(&Path::new(input_directory));
    let master_files = fs::walk_dir(&Path::new(master_directory));

    let mut image_nums = Box::new(HashSet::new());
    let mut duplicates = Box::new(HashSet::new());

    let matcher = Matcher::new();

    match master_files {
        Ok(results) => {
            for file_path in results {
                if file_path.is_file() {
                    let num = matcher.image_number(file_path.filename_str().unwrap());

                    match num {
                        Some(n) => {
                            image_nums.insert(n);
                        },
                        None => {},
                    }
                }
            }
        },
        Err(e) => println!("{}", e),
    }

    match input_files {
        Ok(results) => {
            for file_path in results {
                if file_path.is_file() {
                    let num = matcher.image_number(file_path.filename_str().unwrap());

                    match num {
                        Some(n) => {
                            if image_nums.contains(&n) {
                                if !quiet {
                                    println!("Duplicate: {}", file_path.display());
                                }
                                duplicates.insert(file_path);
                            }
                        },
                        None => {},
                    }
                }
            }
        },
        Err(e) => println!("{}", e),
    }

    println!("{} duplicates found, {} files scanned", duplicates.len(), duplicates.len() + image_nums.len());

    let mut out_file = match matches.opt_str("o") {
        Some(f) => Option::Some(File::create(&Path::new(f)).unwrap()),
        None => Option::None,
    };
    let delete =  matches.opt_present("d");

    for dup in duplicates.iter() {
        if delete {
            fs::unlink(&dup);
        }
        match out_file {
            Some(ref mut f) => {
                f.write_str(format!("{}\n", dup.display()).as_slice());
            },
            _ => { },
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(brief.as_slice()));
}
