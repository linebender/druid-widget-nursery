// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use druid::{Color, Insets, Point, Rect, Size, Vec2};

/// Interpolate between two values
///
/// Interpolate between `self` and `other` where `value` is the
/// position (between 0 and 1). For example, a simple linear
/// interpolation is implemented as: `self + (other - self) * value`
pub trait Interpolate: PartialEq + Clone {
    fn interpolate(&self, other: &Self, value: f64) -> Self;
}

impl Interpolate for f64 {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        self + (other - self) * value
    }
}

//TODO: make this more efficient
impl Interpolate for Color {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        let (r1, g1, b1, a1) = self.as_rgba();
        let (r2, g2, b2, a2) = other.as_rgba();

        Color::rgba(
            r1.interpolate(&r2, value),
            g1.interpolate(&g2, value),
            b1.interpolate(&b2, value),
            a1.interpolate(&a2, value),
        )
    }
}

impl Interpolate for Vec2 {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Vec2::new(
            self.x.interpolate(&other.x, value),
            self.y.interpolate(&other.y, value),
        )
    }
}

impl Interpolate for Point {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Point::new(
            self.x.interpolate(&other.x, value),
            self.y.interpolate(&other.y, value),
        )
    }
}

impl Interpolate for Size {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Size::new(
            self.width.interpolate(&other.width, value),
            self.height.interpolate(&other.height, value),
        )
    }
}

impl Interpolate for Rect {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Rect::from_origin_size(
            self.origin().interpolate(&other.origin(), value),
            self.size().interpolate(&other.size(), value),
        )
    }
}

impl Interpolate for Insets {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        Insets::new(
            self.x0.interpolate(&other.x0, value),
            self.y0.interpolate(&other.y0, value),
            self.x1.interpolate(&other.x1, value),
            self.y1.interpolate(&other.y1, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate> Interpolate for (A, B) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate> Interpolate for (A, B, C) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate, D: Interpolate> Interpolate for (A, B, C, D) {
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
        )
    }
}

impl<A: Interpolate, B: Interpolate, C: Interpolate, D: Interpolate, E: Interpolate> Interpolate
    for (A, B, C, D, E)
{
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
            self.4.interpolate(&other.4, value),
        )
    }
}

impl<
        A: Interpolate,
        B: Interpolate,
        C: Interpolate,
        D: Interpolate,
        E: Interpolate,
        F: Interpolate,
    > Interpolate for (A, B, C, D, E, F)
{
    fn interpolate(&self, other: &Self, value: f64) -> Self {
        (
            self.0.interpolate(&other.0, value),
            self.1.interpolate(&other.1, value),
            self.2.interpolate(&other.2, value),
            self.3.interpolate(&other.3, value),
            self.4.interpolate(&other.4, value),
            self.5.interpolate(&other.5, value),
        )
    }
}
