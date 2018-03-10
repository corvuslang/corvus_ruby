//! A compiled Corvus script

use std::collections::HashMap;
use ruru;
use ruru::{AnyObject, Class, Hash, NilClass, Object, RString, Symbol};
use corvus_core::{Eval, InferredEnv, Scope, SharedNamespace, Syntax, Type};

use classes::corvus_type::CorvusType;
use value::CorvusValue;
use helpers::{build_apply, raise_and_return_nil};

pub struct ScriptData {
  ns: SharedNamespace<CorvusValue>,
  stx: Syntax,
}

wrappable_struct!(ScriptData, ScriptWrapper, WRAPPER);

class!(CorvusScript);
methods!(
  CorvusScript,
  itself,

  fn corvus_script_disallow_new() -> NilClass {
    use ruru::VM;
    VM::raise(Class::from_existing("TypeError"), "call CorvusCompiler#compile to create a CorvusScript");
    NilClass::new()
  }

  fn corvus_script_interpret(globals: Hash) -> AnyObject {
    globals.and_then(|globals| {
      let script_data = itself.get_data(&*WRAPPER);
      let mut scope: Scope<CorvusValue> = Scope::new();
      globals.each(|key, value| {
        key.try_convert_to::<Symbol>().map(|sym| {
          scope.insert(sym.to_string(), CorvusValue::from(value));
        });
      });
      script_data.stx.eval(&script_data.ns, &scope).map_err(|e| ruru::result::Error::TypeError(format!("{}", e))).map(|v| v.to_any_object())
    }).unwrap_or_else(raise_and_return_nil)
  }
);

// Not using macros for this because we can do all kinds of unsafe shit
pub extern "C" fn corvus_script_private_corvus_call(
  argc: ruru::types::Argc,
  argv: *const AnyObject,
  itself: CorvusScript,
) -> AnyObject {
  use error::Error;
  use ruru::result::Error as RError;
  use helpers::rewrite_error;
  let args = ruru::VM::parse_arguments(argc, argv);
  build_apply(args)
    .and_then(|apply| {
      let data = itself.get_data(&*WRAPPER);
      let result = data
        .ns
        .borrow()
        .eval_apply(apply)
        .map(|v| v.to_any_object())
        .map_err(|err| match err {
          Error::Ruru(err) => err,
          Error::Corvus(err) => RError::TypeError(format!("Corvus error: {}", err)),
          Error::Nil => RError::TypeError(format!("nil value passed to nu")),
          Error::Utf8Error(err) => RError::TypeError(format!("nu string: {}", err)),
        })
        .map_err(rewrite_error(|m| format!("eval apply: {}", m)));
      result
    })
    .unwrap_or_else(raise_and_return_nil)
}

impl CorvusScript {
  pub fn new(
    ns: SharedNamespace<CorvusValue>,
    stx: Syntax,
    return_type: Type,
    input_types: InferredEnv,
    ruby_code: String,
  ) -> AnyObject {
    let mut script: AnyObject =
      get_corvus_class!("Script").wrap_data(ScriptData { ns: ns, stx: stx }, &*WRAPPER);
    let code = RString::from(ruby_code);
    script.send("instance_eval", Some(&[code.to_any_object()]));
    script.instance_variable_set("@ruby_code", code);
    script.instance_variable_set("@return_type", CorvusType::new(return_type));
    script.instance_variable_set(
      "@input_types",
      type_env_to_ruby_hash(input_types).to_any_object(),
    );
    script
  }
}

attr_reader!(CorvusScript, corvus_script_input_types, input_types);
attr_reader!(CorvusScript, corvus_script_return_type, return_type);
attr_reader!(CorvusScript, corvus_script_ruby_code, ruby_code);

fn type_env_to_ruby_hash(env: HashMap<String, Type>) -> Hash {
  let mut hash = Hash::new();
  for (k, v) in env {
    hash.store(RString::from(k), CorvusType::new(v));
  }
  hash
}

pub fn init() {
  init_corvus_class!("Script", |class| {
    class.def_self("new", corvus_script_disallow_new);
    class.def("corvus_call", corvus_script_private_corvus_call);
    class.def("call_interpreted", corvus_script_interpret);

    class.def("input_types", corvus_script_input_types);
    class.def("return_type", corvus_script_return_type);
    class.def("ruby_code", corvus_script_ruby_code);
  });
}
