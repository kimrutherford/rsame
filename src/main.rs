extern crate rsame;

use std::path::Path;

fn main() {

    let f1_path = Path::new("test_data/test7/file1");
    let f2_path = Path::new("test_data/test7/file2");

    let matches = rsame::compare_files(f1_path, f2_path);

    for mat in matches {
        println!("{}", mat);
    }
}
