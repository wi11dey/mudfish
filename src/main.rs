use byte_unit::Byte;
use moka::sync::Cache;
use std::error::Error;
use rouille;
use rmp_serde;
use serde::Deserialize;
use std::path::{
    Path,
    PathBuf
};
use std::option::Option;
use std::fs::{
    self,
    File
};
use std::result::Result;
use std::io::{
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
    filters: Option<PathBuf>,

    /// Maximum size of the cache
    #[arg(default_value = "10 MiB")]
    cache_size: Byte,
}

fn load_filters(filters: &Path) -> Result<FilterSet, String> {
    let mut filter_set = FilterSet::new(false);
    for entry in fs::read_dir(filters)
	.map_err(|err| format!(r#"failed to open filter directory "{}": {err}"#, filters.display()))?
    {
	let path = entry
	    .map_err(|err| format!(r#"failed to read filter directory "{}": {err}"#, filters.display()))?
	    .path();
	for line in BufReader::new(
	    File::open(&path)
		.map_err(|err| format!(r#"failed to open filter file "{}": {err}"#, path.display()))?
	).lines()
	{
	    let filter = line
		.map_err(|err| format!(r#"failed to read filter file "{}": {err}"#, path.display()))?;
	    filter_set.add_filter(&filter, ParseOptions::default())
		.map_err(|err| format!(r#"failed to parse filter "{filter}": {err}"#))?;
	}
    }
    return Ok(filter_set);
}

fn engine_internals(engine: &Engine) -> DeserializeFormat {
    return rmp_serde::from_read(&engine.serialize_raw().unwrap()[5..]).unwrap();
}

fn main() {
    let args = Args::parse();

    let engine = args.filters
	.map(|filters| Engine::from_filter_set(load_filters(&filters).unwrap(), true));

    println!("Hello, world!");

    let cache = Cache::builder()
        .weigher(|_key: &String, value: &String| value.len().try_into().unwrap_or(u32::MAX))
        .max_capacity(args.cache_size.as_u64())
        .build();

    rouille::start_server(format!("localhost:{}", args.port), move |request| {
	rouille::proxy::full_proxy(
            request,
            rouille::proxy::ProxyConfig {
                addr: "example.com:80",
                replace_host: Some("example.com".into()),
            },
        ).unwrap()
    });
}
