pub mod ui;

use std::fs::File;
use std::io::*;
use std::path::Path;
use std::iter::Iterator;
use std::collections::HashMap;
use std::fmt;

struct Chunks {
    lines: Vec<String>,
    lookup: HashMap<String, Vec<usize>>,
}

impl Chunks {
    fn make_lookup(lines: &Vec<String>) -> HashMap<String, Vec<usize>> {
        let mut lookup: HashMap<String, Vec<usize>> = HashMap::new();
        for (i,s) in lines.iter().enumerate() {
            let v = lookup.entry(s.clone()).or_insert(Vec::new());
            v.push(i);
        };
        lookup
    }
    fn get_line(self: &Self, index: usize) {
        self.lines.get(index);
    }
    pub fn new(lines: Vec<String>) -> Chunks {
        Chunks {
            lookup: Chunks::make_lookup(&lines),
            lines: lines,
        }
    }
}

pub struct Match {
    start_pos_1: usize,
    end_pos_1: usize,
    start_pos_2: usize,
    end_pos_2: usize
}

impl fmt::Display for Match {
    fn fmt(self: &Self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}..{})->({}..{})",
               self.start_pos_1, self.end_pos_1,
               self.start_pos_2, self.end_pos_2)
    }
}

fn read(path: &Path) -> Chunks {
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                           &why),
        Ok(file) => file
    };


    let reader = BufReader::new(&file);

    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    Chunks::new(lines)

}

pub fn compare_files(path1: &Path, path2: &Path) -> Vec<Match> {
    let chunks1 = read(path1);
    let chunks2 = read(path2);

    return compare(&chunks1, &chunks2);
}

fn matching_lines(l: &String, chunks: &Chunks) -> Vec<usize> {
    match chunks.lookup.get(l) {
        Some(v) => v.clone(),
        None => vec![]
    }
}

fn make_match(chunks1: &Chunks, index1: usize, chunks2: &Chunks, index2: usize) -> Match {
    let mut search_index_1 = index1;
    let mut search_index_2 = index2;

    loop {
        if search_index_1 == 0 || search_index_2 == 0 {
            break;
        }

        let line_1 = chunks1.get_line(search_index_1 - 1);
        let line_2 = chunks2.get_line(search_index_2 - 1);

        if !line_1.eq(&line_2) {
            break;
        }

        search_index_1 -= 1;
        search_index_2 -= 1;
    }

    Match {
        start_pos_1: search_index_1,
        end_pos_1: search_index_2,
        start_pos_2: index2,
        end_pos_2: index2
    }
}

fn compare(chunks1: &Chunks, chunks2: &Chunks) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    for (index1, line) in chunks1.lines.iter().enumerate() {
        let matched_lines = matching_lines(line, chunks2);

        for index2 in matched_lines {
            matches.push(make_match(chunks1, index1, chunks2, index2))
        }
    }

    matches
}
