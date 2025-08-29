use std::cell::{Cell, RefCell};

/// 智能指针
///
/// | 智能指针     | 所有权          | 可变性      | 线程安全 | 使用场景            |
/// | :----------- | :-------------- | :---------- | :------- | :------------------ |
/// | `Box<T>`     | 单一所有者      | 可变/不可变 | 是       | 堆分配、 trait 对象 |
/// | `Rc<T>`      | 多个所有者      | 不可变      | 否       | 单线程共享数据      |
/// | `Arc<T>`     | 多个所有者      | 不可变      | 是       | 多线程共享数据      |
/// | `Cell<T>`    | 单一所有者      | 内部可变    | 是       | 简单的 Copy 类型    |
/// | `RefCell<T>` | 单一所有者      | 内部可变    | 否       | 复杂的内部可变性    |
/// | `Mutex<T>`   | 通常与 Arc 共用 | 内部可变    | 是       | 多线程内部可变性    |

/// Box指针用于在堆上分配内存, 并提供内存管理
fn box_pointer() {
    // 在栈上创建一个很大的数组
    // let huge_array = [0u8; 10_000_000]; // 这可能导致栈溢出

    // 使用 Box 将大数据分配到堆上
    let boxed_array = Box::new([0u8; 10_000_00]);
    println!("数组长度: {}", boxed_array.len());
    // boxed_array 离开作用域时，自动释放堆内存
}

/// Rc指针用于单线程内共享数据.
fn rc_pointer() {
    let data = std::rc::Rc::new(String::from("共享的数据"));

    // 克隆 Rc，增加引用计数，而不是克隆底层数据
    let owner1 = std::rc::Rc::clone(&data);
    let owner2 = std::rc::Rc::clone(&data);

    println!("data: {}", data);
    println!("owner1: {}", owner1);
    println!("owner2: {}", owner2);

    // 查看引用计数
    println!("引用计数: {}", std::rc::Rc::strong_count(&data)); // 输出: 3

    // 当最后一个 Rc 离开作用域时，数据会被清理
}

fn arc_pointer() {
    let data = std::sync::Arc::new(String::from("多线程共享数据"));
    let mut handles = vec![];

    for i in 0..3 {
        // 克隆 Arc 以便移动到线程中
        let data_clone = std::sync::Arc::clone(&data);
        let handle = std::thread::spawn(move || {
            println!("线程 {}: {}", i, data_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("最终数据: {}", data);
}

fn cell_pointer() {
    let counter = Cell::new(0);

    // 即使 counter 是不可变的，我们也能修改其内部值
    for _ in 0..5 {
        let current = counter.get(); // 获取当前值
        counter.set(current + 1);    // 设置新值
    }

    println!("最终计数: {}", counter.get()); // 输出: 5
}

fn mutex_pointer() {
    let data = String::new();
    let lock = std::sync::Mutex::new(data);

    let mut guard = lock.lock().unwrap();
    guard.push_str("abcdefg");
    println!("{:#?}", guard);
}


fn refcell_pointer() {
    let message = RefCell::new(String::from("Hello"));

    // 不可变借用下修改内容
    {
        let mut mutable_borrow = message.borrow_mut();
        mutable_borrow.push_str(", World!");
    } // 可变借用在这里离开作用域

    // 现在可以不可变借用
    let reader = message.borrow();
    println!("{}", reader); // 输出: Hello, World!

    // 运行时检查借用规则：这会 panic!
    // let another_borrow = message.borrow_mut(); // 已经有一个不可变借用存在
}


/// RefCell和Rc组合, 可以构建一个Node结构体, 可以做单线程的树或者图结构

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<std::rc::Rc<Node>>>,
}


fn sample_tree() {
    // 创建几个节点
    let leaf = std::rc::Rc::new(Node {
        value: 3,
        children: RefCell::new(vec![]),
    });

    let branch = std::rc::Rc::new(Node {
        value: 5,
        children: RefCell::new(vec![std::rc::Rc::clone(&leaf)]),
    });

    // 修改 branch 的子节点（即使 branch 是不可变的）
    branch.children.borrow_mut().push(std::rc::Rc::clone(&leaf));

    println!("Branch: {:?}", branch);
    println!("Leaf 的引用计数: {}", std::rc::Rc::strong_count(&leaf)); // 输出: 3
}

fn main() {
    box_pointer();
    rc_pointer();
    arc_pointer();
    cell_pointer();
    refcell_pointer();
    mutex_pointer();
    sample_tree();
}


