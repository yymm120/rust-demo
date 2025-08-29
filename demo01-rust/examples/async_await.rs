#![allow(unused)]

///
/// 通过 async 标记的语法块会被转换成实现了Future特征的状态机。
///
#[cfg(test)]
mod async_test {
    use futures::executor::block_on;
    use std::collections::LinkedList;

    async fn do_something1() {
        println!("do something 1");
    }

    async fn do_something2() {
        do_something3().await;
        println!("do something 2");
    }

    async fn do_something3() {
        println!("do something 3");
    }

    #[test]
    fn async_base() {
        let future1 = do_something1();
        let future2 = do_something2();
        block_on(future1);
    }

    #[test]
    fn test_linked_list() {
        let key_list = LinkedList::from(["language", "platform"]);
        let value_list = LinkedList::from(["rust", "linux"]);

        let mut key_iter = key_list.iter();
        let mut value_iter = value_list.iter();

        #[derive(Debug)]
        struct Return {
            key: String,
            value: String,
        }

        for _ in 0..key_list.len() {
            println!("{:?}", Return {
                key: key_iter.next().unwrap().to_string(),
                value: value_iter.next().unwrap().to_string(),
            })
        }
    }
}
fn main() {}
