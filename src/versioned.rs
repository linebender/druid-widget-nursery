// Copyright 2021 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::ops;

use druid::Data;

/// Data with explicit version.
///
/// This is a wrapper around `Data` that allows to manually mark the `Data` as
/// [`changed`](Versioned::changed).
/// This is useful when data has interior mutablity.
#[derive(Clone)]
pub struct Versioned<T> {
    version: u64,
    data: T,
}

impl<T> Versioned<T> {
    pub fn new(data: T) -> Self {
        Self { version: 0, data }
    }

    /// Mark that data has changed.
    pub fn changed(&mut self) {
        self.version += 1;
    }
}

impl<T> ops::Deref for Versioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> ops::DerefMut for Versioned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Data for Versioned<String> {
    fn same(&self, other: &Self) -> bool {
        self.version == other.version
    }
}
