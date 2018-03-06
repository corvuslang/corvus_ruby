use std::error::Error;
use ruru::{AnyObject, Boolean, NilClass, Object, RString, Symbol, VM};
use ruru::result::{Error as RuruError, Result as RuruResult};
use value::CorvusValue;
use corvus_core::Apply;

pub fn raise_and_return_nil(err: RuruError) -> AnyObject {
    VM::raise(err.to_exception(), err.description());
    NilClass::new().to_any_object()
}

pub fn stringify_key(key: AnyObject) -> RuruResult<String> {
    key.try_convert_to::<Symbol>()
        .map(|s| s.to_string())
        .or_else(|_| key.try_convert_to::<RString>().map(|s| s.to_string()))
}

pub fn rewrite_error<F: FnOnce(String) -> String>(f: F) -> impl FnOnce(RuruError) -> RuruError {
    use ruru::result::Error::*;
    |err: RuruError| match err {
        TypeError(message) => TypeError(f(message)),
        ArgumentError(message) => ArgumentError(f(message)),
    }
}

pub fn truthy(it: AnyObject) -> bool {
    if it.is_nil() {
        return false;
    }
    it.try_convert_to()
        .map(|b: Boolean| b.to_bool())
        .unwrap_or(true)
}

pub fn build_apply(args: Vec<AnyObject>) -> RuruResult<Apply<CorvusValue>> {
    let mut apply: Apply<CorvusValue> = Apply::with_capacity(args.len() / 2);
    let mut i = 1;
    loop {
        let name: Symbol = args[i - 1].try_convert_to()?;
        apply.push_arg(name.to_str(), CorvusValue::from(args[i].clone()));
        i += 2;
        if i >= args.len() {
            break;
        }
    }
    Ok(apply)
}
