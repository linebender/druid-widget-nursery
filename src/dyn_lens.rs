// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A version of Lens that can be made into a trait object

use druid::Lens;

/// A version of Lens that can be made into a trait object.
pub trait DynLens<T, U> {
    fn with_raw(&self, t: &T, f: &mut dyn FnMut(&U));
    fn with_mut_raw(&self, t: &mut T, f: &mut dyn FnMut(&mut U));
}

impl<T, U, L: Lens<T, U>> DynLens<T, U> for L {
    fn with_raw(&self, data: &T, f: &mut dyn FnMut(&U)) {
        Lens::with(self, data, f)
    }

    fn with_mut_raw(&self, data: &mut T, f: &mut dyn FnMut(&mut U)) {
        Lens::with_mut(self, data, f)
    }
}

impl<T, U> dyn DynLens<T, U> {
    pub fn with<R>(&self, data: &T, f: impl FnOnce(&U) -> R) -> R {
        let mut f = Some(f);
        let mut r = None;
        self.with_raw(data, &mut |value| {
            if let Some(f) = f.take() {
                r = Some(f(value));
            }
        });
        r.unwrap()
    }

    pub fn with_mut<R>(&self, data: &mut T, f: impl FnOnce(&mut U) -> R) -> R {
        let mut f = Some(f);
        let mut r = None;
        self.with_mut_raw(data, &mut |value| {
            if let Some(f) = f.take() {
                r = Some(f(value));
            }
        });
        r.unwrap()
    }
}
