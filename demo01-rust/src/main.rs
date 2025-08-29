fn main() {
    let array: [u32; 1] = [0; 1];
    let option_array: [Option<u32>; 1] = [None; 1];


    println!("Array size: {} bytes", std::mem::size_of_val(&array));
    println!("Option array size: {} bytes", std::mem::size_of_val(&option_array));

    // 查看原始内存的二进制表示
    println!("\n=== Raw [0u32; 16] memory ===");
    print_raw_memory(&array);

    println!("\n=== Raw [Some(0u32); 16] memory ===");
    print_raw_memory(&option_array);;

    println!("\n\n");

    string_demo();
}

fn print_raw_memory<T>(data: &T) {
    let size = std::mem::size_of_val(data);
    let ptr = data as *const T as *const u8;

    unsafe {
        for i in 0..size {
            let byte = *ptr.add(i);
            print!("{:08b} ", byte);
            if (i + 1) % 4 == 0 {
                println!(); // 每4字节换行
            }
        }
        println!();
    }
}


fn string_demo() {
   let str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbba".to_string();

    println!("str size {}", size_of_val(&str));
    println!("str ptr {:?}", (&str as *const String));
    println!("str vec size {:?}", str.len());
}










