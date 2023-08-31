extern crate regex;

use std::{fs::File,
          io::*,
          path::Path,
          iter::Iterator,
          collections::{HashSet, HashMap},
          fmt};
use regex::Regex;
#[macro_use] extern crate lazy_static;

struct Chunks {
    lines: Vec<String>,
    lookup: HashMap<String, Vec<usize>>,
}

impl Chunks {
    fn make_lookup(opts: &HashMap<&str, &str>,
                   lines: &Vec<String>) -> HashMap<String, Vec<usize>> {
        let mut lookup: HashMap<String, Vec<usize>> = HashMap::default();
        for (i,s) in lines.iter().enumerate() {
            let fixed_ws_string =
                if opts.contains_key("ignore-whitespace") {
                    WHITESPACE_RE.replace_all(s, " ").to_string()
                } else {
                    s.clone()
                };
            if !opts.contains_key("ignore-blank-lines") ||
                (s.len() > 0 && !WHITESPACE_RE.is_match(s)) {
                    let v = lookup.entry(fixed_ws_string.clone()).or_insert(Vec::new());
                    v.push(i);
                }
        };
        lookup
    }
    fn get_line(self: &Self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }
    pub fn new(opts: &HashMap<&str, &str>, lines: Vec<String>) -> Chunks {
        Chunks {
            lookup: Chunks::make_lookup(opts, &lines),
            lines: lines,
        }
    }
}

lazy_static! {
    static ref WHITESPACE_RE: Regex = Regex::new(r"(\s+)").unwrap();
}

fn squash_whitespace(s: &str) -> String
{
    WHITESPACE_RE.replace_all(s, " ").to_string()
}

fn eq_ignoring_whitespace(str1: &str, str2: &str) -> bool {
    squash_whitespace(str1).eq(&squash_whitespace(str2))
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

fn read(opts: &HashMap<&str, &str>, path: &Path) -> Chunks {
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                           &why),
        Ok(file) => file
    };


    let reader = BufReader::new(&file);

    let lines: Vec<_> = reader.lines().map(|l| l.unwrap()).collect();

    Chunks::new(opts, lines)

}

pub fn compare_files(opts: &HashMap<&str, &str>,
                     path1: &Path, path2: &Path) -> Vec<Match> {
    let chunks1 = read(opts, path1);
    let chunks2 = read(opts, path2);

    return compare(opts, &chunks1, &chunks2);
}

fn chunk_lookup(chunks: &Chunks, key: impl AsRef<str>) -> Vec<usize> {
    match chunks.lookup.get(key.as_ref()) {
        Some(v) => v.clone(),
        None => vec![]
    }
}

fn matching_lines(opts: &HashMap<&str, &str>,
                  l: &str, chunks: &Chunks) -> Vec<usize> {
    if opts.contains_key("ignore-whitespace") {
        chunk_lookup(chunks, squash_whitespace(l))
    } else {
        chunk_lookup(chunks, l)
    }
}

#[derive(PartialEq)]
enum Direction {
    Forward,
    Reverse
}

fn search_out(opts: &HashMap<&str, &str>,
              seen_matching_lines: &mut HashSet<(usize,usize)>,
              chunks1: &Chunks, start_index_1: usize,
              chunks2: &Chunks, start_index_2: usize,
              direction: Direction) -> (usize, usize) {
    let mut search_index_1 = start_index_1;
    let mut search_index_2 = start_index_2;

    loop {
        if direction.eq(&Direction::Reverse) &&
            (search_index_1 == 0 || search_index_2 == 0) {
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

        if opts.contains_key("ignore-whitespace") {
            if !eq_ignoring_whitespace(line_1, line_2) {
                break;
            }
        } else {
            if !line_1.eq(line_2) {
                break;
            }
        }

        search_index_1 = check_index_1;
        search_index_2 = check_index_2;

        seen_matching_lines.insert((search_index_1, search_index_2));
    }

    (search_index_1, search_index_2)
}

fn make_match(opts: &HashMap<&str, &str>,
              seen_matching_lines: &mut HashSet<(usize,usize)>,
              chunks1: &Chunks, index1: usize, chunks2: &Chunks, index2: usize) -> Match {
    let (match_end_1, match_end_2) =
        search_out(opts, seen_matching_lines,
                   chunks1, index1, chunks2, index2, Direction::Forward);
    let (match_start_1, match_start_2) =
        search_out(opts, seen_matching_lines,
                   chunks1, index1, chunks2, index2, Direction::Reverse);

    Match {
        start_pos_1: match_start_1,
        start_pos_2: match_start_2,
        end_pos_1: match_end_1,
        end_pos_2: match_end_2,
    }
}

fn compare(opts: &HashMap<&str, &str>,
           chunks1: &Chunks, chunks2: &Chunks) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    let mut seen_matching_lines: HashSet<(usize, usize)> = HashSet::new();

    for (index1, line) in chunks1.lines.iter().enumerate() {
        let matched_lines = matching_lines(opts, line, chunks2);

        for index2 in matched_lines {
            if !seen_matching_lines.contains(&(index1, index2)) {
                let this_match =
                    make_match(opts, &mut seen_matching_lines,
                               chunks1, index1, chunks2, index2);
                seen_matching_lines.insert((index1, index2));
                matches.push(this_match);
            }
        }
    }

    matches
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use super::Match;
    use super::compare_files;

    fn test_comp(opts: &HashMap<&str, &str>, file1: &str, file2: &str) -> Vec<Match> {
        compare_files(opts, Path::new(file1), Path::new(file2))
    }

    #[test]
    fn test_1() {
        let conf: HashMap<&str, &str> = HashMap::new();

        let res = test_comp(&conf, "test_data/test1/file1", "test_data/test2/file2");

        if res.len() != 2 {
            panic!();
        }
    }
}
