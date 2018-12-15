#[macro_use]
extern crate example_derive;

#[derive(Iterator)]
enum Iter<A, B> {
    A(A),
    B(B),
}

fn return_iter(x: i32) -> impl Iterator<Item = i32> {
    if x < 0 {
        Iter::A(x..=0)
    } else {
        Iter::B(0..x)
    }
}

fn main() {
    let iter = return_iter(-10);
    let iter2 = return_iter(10);
    assert_eq!(iter.fold(0, |sum, x| sum + x), -55);
    assert_eq!(iter2.fold(0, |sum, x| sum + x), 45);
}
