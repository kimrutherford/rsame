extern crate rsame;
extern crate getopts;

use std::path::Path;
use std::env;
use std::collections::HashMap;
use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    let mut conf = HashMap::new();

    opts.optflag("b", "ignore-blank-lines",
                 "ignore empty lines, ie. only whitespace chars");
    opts.optflag("w", "ignore-whitespace",
                 "ignore whitespace differences");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("Invalid options\n{}", f)
    };

    if matches.opt_present("b") {
        conf.insert("ignore-blank-lines", "");
    }
    if matches.opt_present("w") {
        conf.insert("ignore-whitespace", "");
    }

    let files = matches.free;

    let matches = rsame::compare_files(&conf,
                                       Path::new(files.get(0).unwrap()),
                                       Path::new(files.get(1).unwrap()));

    for mat in matches {
        println!("{}", mat);
    }
}
