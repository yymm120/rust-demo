

fn main() {
    let slice1 = "hello"; //
    let slice2 = slice1;
    let mut slice3 = slice1;
    drop_slice1(slice1);
    let slice2 = &slice1[1..2];

    println!("{}", slice1);
    println!("{}", slice2);
}

fn drop_slice1(slice: &str) {
    println!("{}", slice);
}
