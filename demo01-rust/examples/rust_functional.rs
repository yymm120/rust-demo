

fn main() {
    let result = (1..=10)           // 范围迭代器
        .filter(|x| x % 2 == 0)     // 过滤偶数
        .map(|x| x * x)             // 平方
        .take(2)                    // 取前两个
        .sum::<i32>();              // 求和


    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // 函数式编程管道：链式调用
    let result = numbers
        .iter()                      // 转换为迭代器
        .filter(|&x| x % 2 == 0)     // 过滤偶数 (高阶函数)
        .map(|x| x * x)              // 平方 (映射)
        .take(3)                     // 取前3个 (惰性求值)
        .collect::<Vec<i32>>();      // 收集结果

    println!("{:?}", result); // 输出: [4, 16, 36]

    // 函数组合: (f ∘ g)(x) = f(g(x))
    let add_one = |x| x + 1;
    let square = |x| x * x;
    let add_then_square = compose(square, add_one);

    println!("(2 + 1)² = {}", add_then_square(2)); // 输出: 9

    // 不可变性和纯函数
    let sum: i32 = numbers.iter().sum();
    println!("总和: {}", sum); // 同样的输入总是得到同样的输出

    // Option 和 Result 的函数式处理
    let maybe_number = Some(42);
    let transformed = maybe_number
        .map(|x| x * 2)              // 如果有值，就转换
        .and_then(|x| if x > 50 { Some(x) } else { None }) // 链式处理
        .unwrap_or(0);               // 提供默认值

    println!("转换结果: {}", transformed); // 输出: 0 (因为 84 > 50 为 true)
}

// 函数组合：实现 f ∘ g
fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(B) -> C,
    G: Fn(A) -> B,
{
    move |x| f(g(x)) // 闭包返回新函数
}

// 递归函数 - 函数式编程的另一个特征
fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1), // 递归调用
    }
}

// 使用模式匹配的函数
fn describe_number(n: i32) -> String {
    match n {
        n if n < 0 => "负数".to_string(),
        0 => "零".to_string(),
        1..=9 => "个位数".to_string(),
        _ => "多位数".to_string(),
    }

}


