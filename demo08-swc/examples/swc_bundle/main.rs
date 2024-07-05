#![allow(dead_code)]
#![allow(unused)]

mod bundle;
use std::sync::Arc;

use swc_common::{sync::Lrc, SourceMap, FilePathMapping};

use bundle::example_swc_bundler_entry;

mod path;
use path::example_swc_path_entry;


// cargo watch -c -q -w examples/swc_bundle -x "run --example swc_bundle examples/swc_bundle/assets/main.js"
fn main() {
    example_swc_bundler_entry();
    // println!("--------------------------");
    // example_swc_path_entry();

    // my test func
    test_some();
}

fn test_some() {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
}