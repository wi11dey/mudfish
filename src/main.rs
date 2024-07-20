use rmp_serde;
use serde::Deserialize;
use std::path::PathBuf;
use std::option::Option;
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
    lists::{FilterSet, ParseOptions},
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::vec::Vec;

type Hash = u64;
type NetworkFilterMask = u32;

#[derive(Deserialize)]
enum FilterPart {
    Empty,
    // Simple(String),
    Simple(()),
    // AnyOf(Vec<String>),
    AnyOf(()),
}

#[derive(Deserialize)]
struct NetworkFilterV0DeserializeFmt {
    _mask: NetworkFilterMask,
    _filter: FilterPart,
    _opt_domains: Option<Vec<Hash>>,
    _opt_not_domains: Option<Vec<Hash>>,
    _redirect: Option<String>,
    _hostname: Option<String>,
    _csp: Option<String>,
    _bug: Option<u32>,
    _tag: Option<String>,
    _raw_line: Option<String>,
    _id: Hash,
    _opt_domains_union: Option<Hash>,
    _opt_not_domains_union: Option<Hash>,
}

#[derive(Deserialize)]
struct NetworkFilterListV0DeserializeFmt {
    _filter_map: HashMap<Hash, Vec<NetworkFilterV0DeserializeFmt>>,
}

#[derive(Deserialize)]
struct LegacyRedirectResource {
    _content_type: String,
    _data: String,
}

#[derive(Deserialize)]
struct LegacyRedirectResourceStorage {
    _resources: HashMap<String, LegacyRedirectResource>,
}

#[derive(Deserialize)]
struct DeserializeFormat {
    _csp: NetworkFilterListV0DeserializeFmt,
    _exceptions: NetworkFilterListV0DeserializeFmt,
    _importants: NetworkFilterListV0DeserializeFmt,
    _redirects: NetworkFilterListV0DeserializeFmt,
    _filters_tagged: NetworkFilterListV0DeserializeFmt,
    _filters: NetworkFilterListV0DeserializeFmt,
    _generic_hide: NetworkFilterListV0DeserializeFmt,

    _tagged_filters_all: Vec<NetworkFilterV0DeserializeFmt>,

    _enable_optimizations: bool,

    _resources: LegacyRedirectResourceStorage,

    simple_class_rules: HashSet<String>,
    simple_id_rules: HashSet<String>,
    complex_class_rules: HashMap<String, Vec<String>>,
    complex_id_rules: HashMap<String, Vec<String>>,

    // specific_rules: LegacyHostnameRuleDb,

    // misc_generic_selectors: HashSet<String>,

    // _scriptlets: LegacyScriptletResourceStorage,
}

#[derive(Parser)]
struct Args {
    /// Port to start HTTP proxy on
    #[arg(long, env)]
    port: u16,

    /// Directory with Adblock-format filter lists
    filters: Option<PathBuf>
}

fn load_filters(filters: PathBuf) -> FilterSet {
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

fn engine_internals(engine: &Engine) -> DeserializeFormat {
    let serialized = engine.serialize_raw().expect("");
    return rmp_serde::from_read(&serialized[5..]).expect("");
}

fn main() {
    let args = Args::parse();

    if let Some(filters) = args.filters {
	let engine = Engine::from_filter_set(load_filters(filters), true);
	engine_internals(&engine);
    }

    println!("Hello, world!");
}
