use druid::Env;
use crate::multi_value::INDENT;

pub fn configure_env<T>(env: &mut Env, _: &T) {
    env.set(INDENT, 30.0);
}