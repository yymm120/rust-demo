
/**
 * Rust 方法往往跟结构体, 枚举,特征 一起使用

 * Rust 使用impl 定义方法
 */

struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}

impl Circle {
    // 关联函数 (构造)
    // 方法中,没有self参数的方法称之为关联函数
    // Rust中有一个约定俗成的规则, 使用new来作为构造器的名称
    // 关联函数是一个函数,所以需要用::来调用
    fn new(x: f64, y: f64, radius: f64) -> Circle {
        Circle {
            x,
            y,
            radius,
        }
    }
    // 普通方法, (&self是self: &self的缩写, self: &self 是 circle: &Circle的缩写)
    // self 所有权转移
    // &self 不可变借用, 使用&self的理由, 不想获取所有权, 也无需修改它
    // &mut self 可变借用, 希望修改当前结构体
    fn area(&self) -> f64 {
        std::f64::consts::PI * (self.radius * self.radius)
    }

    // Getter
    pub fn width(&self) -> f64 {
        return self.x;
    }
}


fn main() {
    let rect1 = Circle::new(30f64, 50f64, 1f64);
    // rust具有自动解引用, 所以下面代码是等价的
    rect1.width();
    (&rect1).width();
    //
    println!("Hello, world!");
}


fn test_function1() {}
