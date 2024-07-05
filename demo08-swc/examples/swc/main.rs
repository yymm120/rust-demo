#![allow(unused)]
#![allow(unused_imports)]

mod transform;
use transform::transform_01;

mod transform_error;
use transform_error::transform_02_error;

mod minify;
use minify::transform_03_minify;

// cargo watch -c -q -w examples/swc -x "run --example swc"
pub fn main() {

    // 非源码行
    println!("{:?}", std::env::current_dir());

    // transform_01();
    transform_03_minify();
    // transform_02_error();

}