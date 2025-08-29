#![allow(unused)]

use std::marker::PhantomPinned;
use std::any::type_name;
use std::pin::{pin, Pin};
use std::ptr;
use std::ptr::NonNull;

struct SelfRefPined {
    data: i32,
    self_ptr: *const i32,// 一个指针，指向自己的 `data` 字段
    _pin: PhantomPinned, // 标记为 !Unpin
}
#[test]
fn unpin() {
    // 创建并初始化一个固定在堆上的自引用结构
    let pined = SelfRefPined::new(42);
    println!("pined.data: {}", pined.data); // 输出: 42
    println!("pined.self_ptr.data: {}", pined.as_ref().get_data_via_ptr()); // 输出: 42
    // 移动 Box本身是安全的，因为堆上数据的地址不变
    let moved_pin = pined;
    println!("After moving the Box, data is still valid: {}", moved_pin.as_ref().get_data_via_ptr()); // 输出: 42
}

fn main() {
}

impl SelfRefPined {
    fn new(data: i32) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(Self {
            data,
            self_ptr: ptr::null(),
            _pin: PhantomPinned,
        });

        // 在固定的数据上执行初始化
        let self_ptr: *const i32 = &boxed.data;

        // 初始化，数据已被 Pin 在堆上，地址不会变
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).self_ptr = self_ptr;
        }

        boxed
    }

    fn get_data_via_ptr(self: Pin<&Self>) -> i32 {
        // 安全原因：我们确信 `self_ptr` 在初始化后有效且不会失效
        unsafe { *self.self_ptr }
    }
}