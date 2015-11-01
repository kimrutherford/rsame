use std::fs::File;
use std::io::*;
use std::path::Path;
use std::iter::Iterator;
use std::collections::HashMap;
use std::collections::HashSet;
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
    fn get_line(self: &Self, index: usize) -> Option<&String> {
        self.lines.get(index)
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

impl Clone for Match {
    fn clone(self: &Self) -> Match {
        return Match {
            start_pos_1: self.start_pos_1,
            end_pos_1: self.end_pos_1,
            start_pos_2: self.start_pos_2,
            end_pos_2: self.end_pos_2,
        }
    }
}

impl fmt::Display for Match {
    fn fmt(self: &Self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}..{})->({}..{})",
               self.start_pos_1 + 1, self.end_pos_1 + 1,
               self.start_pos_2 + 1, self.end_pos_2 + 1)
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

enum Direction {
    Forward,
    Reverse
}

fn search_out(seen_matching_lines: &mut HashSet<(usize,usize)>,
              chunks1: &Chunks, start_index_1: usize,
              chunks2: &Chunks, start_index_2: usize,
              direction: Direction) -> (usize, usize) {
    let mut search_index_1 = start_index_1;
    let mut search_index_2 = start_index_2;

    loop {
        if search_index_1 == 0 || search_index_2 == 0 {
            break;
        }

        let check_index_1 = match direction {
            Direction::Forward => search_index_1 + 1,
            Direction::Reverse => search_index_1 - 1
        };
        let line_1 = match chunks1.get_line(check_index_1) {
            Some(l) => l,
            None => break
        };

        let check_index_2 = match direction {
            Direction::Forward => search_index_2 + 1,
            Direction::Reverse => search_index_2 - 1
        };
        let line_2 = match chunks2.get_line(check_index_2) {
            Some(l) => l,
            None => break
        };

        if !line_1.eq(line_2) {
            break;
        }

        search_index_1 = check_index_1;
        search_index_2 = check_index_2;

        seen_matching_lines.insert((search_index_1, search_index_2));
    }

    return (search_index_1, search_index_2);
}

fn make_match(seen_matching_lines: &mut HashSet<(usize,usize)>,
              chunks1: &Chunks, index1: usize, chunks2: &Chunks, index2: usize) -> Match {
    let (match_start_1, match_start_2) =
        search_out(seen_matching_lines,
                   chunks1, index1, chunks2, index2, Direction::Forward);
    let (match_end_1, match_end_2) =
        search_out(seen_matching_lines,
                   chunks1, index1, chunks2, index2, Direction::Reverse);

    Match {
        start_pos_1: match_start_1,
        start_pos_2: match_start_2,
        end_pos_1: match_end_1,
        end_pos_2: match_end_2,
    }
}

fn compare(chunks1: &Chunks, chunks2: &Chunks) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    let mut seen_matching_lines: HashSet<(usize, usize)> = HashSet::new();

    for (index1, line) in chunks1.lines.iter().enumerate() {
        let matched_lines = matching_lines(line, chunks2);

        for index2 in matched_lines {
            if !seen_matching_lines.contains(&(index1, index2)) {
                let this_match =
                    make_match(&mut seen_matching_lines, chunks1, index1, chunks2, index2);
                seen_matching_lines.insert((index1, index2));
                matches.push(this_match);
            }
        }
    }

    matches
}
