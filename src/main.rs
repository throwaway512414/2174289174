use std::fs::File;

use randomlib::run;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let input_file = args.get(1).expect("Path to input file to be provided");

    let f = File::open(input_file).expect("Input file to exist");

    if let Err(e) = run(f, std::io::stdout()) {
        println!("{}", e);
    }
}
