use ruru;
use ruru::result::Error as RError;
use ruru::{AnyObject, Array, Boolean, Class, NilClass, Object, Proc, Symbol};

use corvus_core::{Apply, Namespace, SharedNamespace};

use helpers::{raise_and_return_nil, rewrite_error};
use value::CorvusValue;
use error::Error;
use classes::corvus_type::CorvusType;
use classes::corvus_signature::build_signature;
use classes::corvus_args::CorvusArgs;

wrappable_struct!(SharedNamespace<CorvusValue>, NamespaceWrapper, WRAPPER);

class!(CorvusNamespace);
verify_with_class_name!(CorvusNamespace, "Namespace");
methods!(
  CorvusNamespace,
  itself,

  fn corvus_namespace_new() -> AnyObject {
    Namespace::new_with_prelude().map_err(RError::TypeError).map(|ns: Namespace<CorvusValue>| ns.into_shared()).map(|ns| {
      get_corvus_class!("Namespace").wrap_data(ns, &*WRAPPER)
    }).unwrap_or_else(raise_and_return_nil)
  }

  fn corvus_namespace_define(
    args: Array,
    return_type: CorvusType,
    total: Boolean,
    rproc: Proc
  ) -> AnyObject {
    rproc.and_then(|rproc| {
      let signature = build_signature(args?, return_type?, total?)?;
      let ns = itself.get_data(&*WRAPPER);
      let callback = Box::new({
        let ns = ns.clone();
        move |args: Apply<CorvusValue>| {
          // todo rb_protect ??
          let proc_result = rproc.call(Some(&[CorvusArgs::wrap(ns.clone(), args).to_any_object()]));
          Ok(CorvusValue::from(proc_result))
        }
      });
      ns.borrow_mut().insert(signature, callback).map_err(RError::TypeError)?;
      Ok(NilClass::new().to_any_object())
    }).unwrap_or_else(raise_and_return_nil)
  }

  fn corvus_namespace_corvus_call(args: Array) -> AnyObject {
    args.map(|args| args.into_iter().collect()).and_then(|args: Vec<AnyObject>| {
      let ns = itself.get_data(&*WRAPPER);
      let apply = build_apply(args).map_err(rewrite_error(|m| format!("build apply: {}", m)))?;
      ns.borrow().eval_apply(apply).map(|v| v.to_any_object()).map_err(|err| match err {
        Error::Ruru(err) => err,
        Error::Corvus(err) => RError::TypeError(format!("Corvus error: {}", err)),
        Error::Nil => RError::TypeError(format!("nil value passed to nu")),
        Error::Utf8Error(err) => RError::TypeError(format!("nu string: {}", err)),
      }).map_err(rewrite_error(|m| format!("eval apply: {}", m)))
    }).unwrap_or_else(raise_and_return_nil)
  }
);

impl CorvusNamespace {
  pub fn clone_rc(&self) -> SharedNamespace<CorvusValue> {
    self.get_data(&*WRAPPER).clone()
  }
}

pub fn init() {
  init_corvus_class!("Namespace", |class| {
    class.def_self("new", corvus_namespace_new);
    class.def("define", corvus_namespace_define);
    class.def("corvus_call", corvus_namespace_corvus_call);
  });
}

fn build_apply(args: Vec<AnyObject>) -> ruru::result::Result<Apply<CorvusValue>> {
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
