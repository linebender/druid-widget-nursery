// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "derive")]

use std::{fmt::Debug, marker::PhantomData};

use druid_widget_nursery::prism::Prism;

#[derive(Clone, Prism)]
enum MyOption<T> {
    Some(T),
    None,
}

#[derive(Clone, Prism)]
enum CLike {
    A,
    B,
    C,
}

#[derive(Clone, Prism)]
enum Complex {
    First,
    Second(),
    Third(u32),
    Fourth(String, Box<Complex>),
}

#[derive(Clone, Prism)]
enum LotsOfGenerics<T, U: Debug>
where
    T: Clone,
    (T, U): Clone,
{
    V1,
    V2(T),
    V3(PhantomData<T>, Box<(U, U)>),
}
