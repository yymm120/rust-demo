
fn multiple_instructions() {
    let mut x: i32 = 10;
    let mut y: i32 = 20;

    unsafe {
        std::arch::asm!(
        "mov eax, {0}",   // 将x移动到eax
        "add eax, {1}",   // eax = eax + y
        "mov {0}, eax",   // 将结果移回x
        inout(reg) x,
        in(reg) y,
        );
    }

    println!("x + y = {}", x); // 输出: 30
}

fn atomic_increment() {
    let mut counter: i32 = 0;

    unsafe {
        std::arch::asm!(
        "lock add dword ptr [{0}], 1", // 原子加1
        in(reg) &counter,
        );
    }

    println!("Atomic counter: {}", counter); // 输出: 1
}

fn memory_operations() {
    let mut value: u32 = 42;

    unsafe {
        std::arch::asm!(
        "inc dword ptr [{0}]", // 对内存地址的值加1
        in(reg) &value,        // 传入内存地址
        );
    }

    println!("Value after inc: {}", value); // 输出: 43
}

fn main() {
    unsafe {
        std::arch::asm!("nop"); // 最简单的汇编指令：空操作
    }

    let mut x: i32 = 5;
    let mut y: i32 = 3;

    unsafe {
        std::arch::asm!(
        "add {0}, {1}", // 加法指令
        inout(reg) y,   // 输入输出寄存器
        in(reg) x,      // 输入寄存器
        );
    }
    println!("-- {}", y);

    memory_operations();

    atomic_increment();
}
