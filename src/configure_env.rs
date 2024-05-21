// Copyright 2022 the Druid Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::multi_value::INDENT;
use druid::Env;

pub fn configure_env<T>(env: &mut Env, _: &T) {
    env.set(INDENT, 30.0);
}
