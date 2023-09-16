// SPDX-License-Identifier: Apache-2.0 OR MIT

use example_derive::Iterator;

#[derive(Iterator)]
enum EnumWZV {} //~ ERROR may not be used on enums without variants

#[derive(Iterator)]
enum EnumWD {
    A = 2, //~ ERROR may not be used on enums with discriminants
    B,
}

#[derive(Iterator)]
enum EnumWZF1<B> {
    A, //~ ERROR may not be used on enums with variants with zero fields
    B(B),
}

#[derive(Iterator)]
enum EnumWZF2<B> {
    A(), //~ ERROR may not be used on enums with variants with zero fields
    B(B),
}

#[derive(Iterator)]
enum EnumWMF<A, B> {
    A(A),
    B(A, B), //~ ERROR may not be used on enums with variants with multiple fields
}

#[derive(Iterator)]
enum EnumWNF<A, B> {
    A { x: A }, //~ ERROR may not be used on enums with variants with named fields
    B(B),
}

fn main() {}
