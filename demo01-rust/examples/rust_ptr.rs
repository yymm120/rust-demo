

fn main() {
    let x = 42;

    // 1. 获取裸指针（这是安全的操作）
    let raw_ptr: *const i32 = &x as *const i32; // 不可变裸指针
    let raw_ptr_mut: *mut i32 = &x as *const i32 as *mut i32; // 可变裸指针（转换）

    // 指针必须是0x4的倍数, 如果是随意写一个数, 会得到经典内存错误:panic Segmentation fault (core dumped）
    let abc: *mut i32 = 400000i32 as *mut i32;

    // 2. 解引用裸指针（这是不安全的，必须放在 unsafe 块中）
    unsafe {
        println!("x = {}", *raw_ptr); // 输出: x = 42
        println!("x = {}", *raw_ptr_mut); // 输出: x = 42
        // println!("x = {}", *abc); // 输出: Segmentation fault (core dumped)
    }


    // 修改
    let mut x = 42;
    println!("修改前: {}", x);

    // 获取可变裸指针
    let raw_ptr_mut: *mut i32 = &mut x as *mut i32;

    // 通过指针修改值
    unsafe {
        *raw_ptr_mut = 100; // 解引用并赋值
    }

    println!("修改后: {}", x); // 输出: 修改后: 100



    //展示裸指针的典型操作：检查空指针和指针算术。
    let arr = [1, 2, 3, 4, 5];

    // 获取数组第一个元素的指针
    let ptr: *const i32 = &arr[0] as *const i32;

    unsafe {
        // 1. 检查是否为空指针（虽然这里不可能是）
        if ptr.is_null() {
            println!("指针是空的！");
        } else {
            // 2. 指针算术：访问下一个元素
            let next_ptr = ptr.add(1); // 相当于 ptr.offset(1)
            println!("第一个元素: {}", *ptr);      // 输出: 1
            println!("第二个元素: {}", *next_ptr); // 输出: 2

            // 3. 遍历数组
            for i in 0..arr.len() {
                let current_ptr = ptr.add(i);
                println!("arr[{}] = {}", i, *current_ptr);
            }
        }
    }


    // 与 Box 一起使用
    // 在堆上分配数据
    let boxed = Box::new(42);

    // 将 Box 转换为裸指针（这会消耗 Box，防止双重释放）
    let raw_ptr: *const i32 = Box::into_raw(boxed);

    unsafe {
        println!("堆上的值: {}", *raw_ptr); // 输出: 42

        // 将裸指针重新转换为 Box，以便正确释放堆内存
        let _boxed_again = Box::from_raw(raw_ptr as *mut i32);
        // _boxed_again 在这里离开作用域，内存被自动释放
    }
    // 注意：如果不转换回 Box，会导致内存泄漏！
    // 裸指针没有Drop实现, 不会清理堆内存. Box具有Drop, 离开作用域会自动清理堆内存.
}


