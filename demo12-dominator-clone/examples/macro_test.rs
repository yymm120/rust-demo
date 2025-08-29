
// use demo12_dominator_clone::{class};

use std::{collections::HashMap, hash::Hash};



fn testa(class_name: &str, key: &str, value: &str) {

}

fn testb(class_name: &str, entry: HashMap<&str, &str>) {

}


#[macro_export]
macro_rules! __stringify_for_vec {
    ($($str:tt)*) => {{
        let result: Vec<&str> = ::std::stringify!($($str,)*).split(",").map(|str| str.trim()).filter(|str| str.to_string() != "").collect();
        result
    }};
}



/// .class_name { background: "white"; }
#[macro_export]
macro_rules! class_test {
    (. $class_name:tt { $key:tt: $value:tt ; } ) => {
        (r".class_name { background: 'white'");
        testa(::std::stringify!($class_name), ::std::stringify!($key), $value);
    };

    (. $class_name:tt { $( $key:tt: $value:tt ; )* } ) => {{
        (r".class_name { background: 'white'; color: 'black';");
        let result: HashMap<&str, &str> = __stringify_for_vec!($($key)*).into_iter().zip(__stringify_for_vec!($($value)*).into_iter()).collect();
        result
    }};
}


fn main() {
    let result = class_test!{
        .hot_b {
            background: "white";
            color: "black";
        }
    };
    println!("{:?}", result);
}