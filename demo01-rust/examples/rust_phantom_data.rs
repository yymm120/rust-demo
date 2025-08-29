#![allow(unused)]

//! 泛型可以约束使用者, 但定义泛型就必须使用泛型, 这是编译器规定的.
//!
//! 为了约束使用者, 但又不实际使用泛型, 就可以利用PhantomData虚假类型
//!
//! PhantomData<T> 利用虚假的泛型来约束使用者, 而实际上并没有这个类型.

use std::marker::PhantomData;

struct Handle<T> {
    id: i32,
    _type: PhantomData<T>, // 用泛型 T 来标记句柄的类型
}

// 为特定类型实现方法
impl<T> Handle<T> {
    fn new(id: i32) -> Self {
        Handle {
            id,
            _type: PhantomData,
        }
    }
}

fn process_int(handle: Handle<i32>) { /* ... */ }
fn process_string(handle: Handle<String>) { /* ... */ }

fn main() {
    let int_handle = Handle::<i32>::new(1);   // 这是一个 i32 句柄
    let string_handle = Handle::<String>::new(2); // 这是一个 String 句柄

    process_int(int_handle);       // 正确
    // process_int(string_handle); // 错误！类型不匹配：Expected `Handle<i32>`, found `Handle<String>`
}



