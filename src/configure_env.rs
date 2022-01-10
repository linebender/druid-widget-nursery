use crate::multi_value::INDENT;
use druid::Env;

pub fn configure_env<T>(env: &mut Env, _: &T) {
    env.set(INDENT, 30.0);
}
