use ruru::{AnyObject, Array, Class, NilClass, Object, RString};
use ruru::result::Error;

use corvus_core::{Apply, INamespace, SharedNamespace};

use helpers::raise_and_return_nil;
use value::CorvusValue;

pub struct Args {
    ns: SharedNamespace<CorvusValue>,
    apply: Apply<CorvusValue>,
}

wrappable_struct!(Args, ArgsWrapper, WRAPPER);

class!(CorvusArgs);

methods!(
    CorvusArgs,
    itself,

    fn ruby_args_get(name: RString) -> AnyObject {
        name.and_then(|name| {
            let data = itself.get_data(&*WRAPPER);
            let ns = data.ns.borrow();
            let signature = ns.get_signature(data.apply.func_name()).ok_or_else(|| {
                Error::TypeError(format!("function `{}` is not defined", data.apply.func_name()))
            })?;

            let arg = signature.arg(name.to_str()).ok_or_else(|| {
                Error::TypeError(format!("`{}` is not a valid argument name", name.to_str()))
            })?;

            let mut values = data.apply.iter()
                .filter_map(|&(ref this_name, ref val)| {
                    if this_name == name.to_str() { Some(val.to_any_object()) } else { None }
                });

            if arg.variadic {
                let array: Array = values.collect();
                Ok(array.to_any_object())
            } else {
                if let Some(first) = values.next() {
                    Ok(first)
                } else if arg.required {
                    Err(Error::TypeError(format!("missing required argument `{}`", name.to_str())))
                } else {
                    Ok(NilClass::new().to_any_object())
                }
            }
        }).unwrap_or_else(raise_and_return_nil)
    }
);


impl CorvusArgs {
    pub fn wrap(ns: SharedNamespace<CorvusValue>, apply: Apply<CorvusValue>) -> Self {
        let data = Args {
            ns: ns,
            apply: apply,
        };
        get_corvus_class!("Args").wrap_data(data, &*WRAPPER)
    }
}

pub fn init() {
    init_corvus_class!("Args", |class| {
        class.def("[]", ruby_args_get);
    });
}
