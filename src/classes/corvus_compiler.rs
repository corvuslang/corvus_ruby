use std::iter::empty;
use ruru::{AnyObject, Class, Object, RString};
use ruru::result::Error;

use corvus_core::{parse, type_of, ParseRule};

use emitter;
use helpers::raise_and_return_nil;
use classes::corvus_namespace::CorvusNamespace;
use classes::corvus_script::CorvusScript;

class!(CorvusCompiler);

methods!(
  CorvusCompiler,
  itself,

  fn corvus_compiler_compile(src: RString) -> AnyObject {
    src.and_then(|src| {
      let corvus_ns: CorvusNamespace = itself.instance_variable_get("@ns").try_convert_to()?;
      let ns = corvus_ns.clone_rc();
      let (stx, ty, inferred_env) = {
        let ns = ns.try_borrow().map_err(|e| Error::TypeError(format!("{}", e)))?;
        let stx = parse(&*ns, ParseRule::script, src.to_str()).map_err(|e| Error::ArgumentError(format!("{}", e)))?;
        let (ty, inferred_env) = type_of(&*ns, empty(), &stx).map_err(|err| Error::TypeError(format!("{}", err)))?;
        (stx, ty, inferred_env)
      };
      let ruby_code = emitter::emit(&stx).map_err(|err| Error::TypeError(format!("emit: {:?}", err)))?;
      Ok(CorvusScript::new(ns, stx, ty, inferred_env, ruby_code))
    }).unwrap_or_else(raise_and_return_nil)
  }
);

pub fn init() {
  Class::from_existing("Corvus")
    .get_nested_class("Compiler")
    .def("compile", corvus_compiler_compile);
}
