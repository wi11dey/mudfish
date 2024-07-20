use std::path::PathBuf;
use std::fs::{
    self,
    File
};
use std::io::{
    Result,
    BufRead,
    BufReader
};
use clap::Parser;
use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions}
};

#[derive(Parser)]
struct Args {
    /// Port to start HTTP proxy on
    #[arg(long, env)]
    port: u16,

    /// Directory with Adblock-format filter lists
    filters: PathBuf
}

fn load_filters(filters: &PathBuf) -> FilterSet {
    let mut filter_set = FilterSet::new(false);
    // Prefer to panic rather than skip some requested filters:
    fs::read_dir(filters)
	.expect("failed to open filter directory {filters}")
	.map(Result::unwrap)
	.map(|entry| entry.path())
	.flat_map(|path| BufReader::new(File::open(path).expect("failed to open filter list {path}"))
		  .lines()
		  .map(|filter| filter.expect("failed to read filter list {path}")))
	.for_each(|filter| filter_set.add_filter(&filter, ParseOptions::default())
		  .expect("failed to parse filter {filter}"));
    return filter_set;
}

fn main() {
    let args = Args::parse();

    Engine::from_filter_set(load_filters(&args.filters), true);

    println!("Hello, world!");
}
