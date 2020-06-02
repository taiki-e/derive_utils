#![warn(rust_2018_idioms, single_use_lifetimes)]

use example_derive::*;

#[derive(Iterator, ExactSizeIterator, Future)]
enum Enum<A, B> {
    A(A),
    B(B),
}

fn return_iter(x: i16) -> impl ExactSizeIterator<Item = i16> {
    if x < 0 { Enum::A(x..=0) } else { Enum::B(0..x) }
}

fn main() {
    let iter = return_iter(-10);
    let iter2 = return_iter(10);
    assert_eq!(iter.len(), 11);
    assert_eq!(iter2.len(), 10);
    assert_eq!(iter.fold(0, |sum, x| sum - x), 55);
    assert_eq!(iter2.fold(0, |sum, x| sum - x), -45);
}
