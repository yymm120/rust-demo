use std::alloc::{alloc, dealloc, Layout};

fn basic_alloc() {
    // 1. 定义内存布局：我们想要分配一个 i32 的内存空间
    let layout = Layout::new::<i32>();

    unsafe {
        // 2. 分配内存（相当于 malloc）
        // alloc 返回一个 *mut u8 指针，指向分配的内存块
        let ptr = alloc(layout) as *mut i32;

        // 检查分配是否成功
        if ptr.is_null() {
            panic!("内存分配失败！");
        }

        // 3. 在分配的内存中写入值（相当于 *ptr = 42）
        ptr.write(42);

        // 4. 读取值（相当于 *ptr）
        let value = ptr.read();
        println!("分配的内存中的值: {}", value); // 输出: 42

        // 5. 手动释放内存（相当于 free）
        // 必须使用与分配时相同的 Layout
        dealloc(ptr as *mut u8, layout);

        // 注意：在此之后，ptr 就成了悬垂指针，不应该再被使用
        // println!("{}", ptr.read()); // 危险！未定义行为
    }
}

fn alloc_vec() {
    let size = 5; // 数组大小
    let layout = Layout::array::<i32>(size).unwrap(); // 为 5 个 i32 分配内存

    unsafe {
        // 分配内存
        let ptr = alloc(layout) as *mut i32;

        if ptr.is_null() {
            panic!("内存分配失败！");
        }

        // 初始化数组
        for i in 0..size {
            ptr.add(i).write((i * 10) as i32);
        }

        // 读取和打印数组
        println!("手动分配的数组:");
        for i in 0..size {
            let value = ptr.add(i).read();
            println!("arr[{}] = {}", i, value);
        }

        // 输出:
        // arr[0] = 0
        // arr[1] = 10
        // arr[2] = 20
        // arr[3] = 30
        // arr[4] = 40

        // 释放内存
        dealloc(ptr as *mut u8, layout);
    }
}

fn main() {
    basic_alloc();
}


