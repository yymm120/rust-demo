use futures::executor::block_on;


async fn hello() {
    println!("hello");
}

async fn invoke() {
    hello().await
}

fn main() {

    block_on(invoke())
}