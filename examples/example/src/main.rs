// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(dead_code)]

use example_derive::{ExactSizeIterator, Iterator, MyTrait1, MyTrait2};

#[derive(Iterator, ExactSizeIterator, MyTrait1, MyTrait2)]
enum Enum<A, B> {
    A(A),
    B(B),
}

#[derive(Iterator, ExactSizeIterator, MyTrait1, MyTrait2)]
enum Enum2<A> {
    A(A),
    B(A),
}

fn return_iter(x: i16) -> impl ExactSizeIterator<Item = i16> {
    if x < 0 {
        Enum::A(x..=0)
    } else {
        Enum::B(0..x)
    }
}

trait MyTrait1 {
    type Assoc1;
    type Assoc2;
}

trait MyTrait2: MyTrait1 {}

fn main() {
    let iter = return_iter(-10);
    let iter2 = return_iter(10);
    assert_eq!(iter.len(), 11);
    assert_eq!(iter2.len(), 10);
    assert_eq!(iter.fold(0, |sum, x| sum - x), 55);
    assert_eq!(iter2.fold(0, |sum, x| sum - x), -45);
}
