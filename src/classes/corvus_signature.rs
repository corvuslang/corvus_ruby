use ruru::{AnyObject, Array, Boolean, Class, Hash, Object, RString, Symbol};

use corvus_core::signature::{Argument, Signature};

use helpers::{raise_and_return_nil, truthy};
use classes::corvus_type::CorvusType;

wrappable_struct!(Signature, SignatureWrapper, WRAPPER);

lazy_static!(
  static ref SYM_NAME: Symbol = Symbol::new("name");
  static ref SYM_TYPE: Symbol = Symbol::new("type");
  static ref SYM_OPTIONAL: Symbol = Symbol::new("required");
  static ref SYM_VARIADIC: Symbol = Symbol::new("variadic");
);

class!(CorvusSignature);
verify_with_class_name!(CorvusSignature, "Signature");
methods!(
  CorvusSignature,
  itself,

  fn corvus_signature_new(args: Array, return_type: CorvusType, total: Boolean) -> AnyObject {
    Ok(()).and_then(|_| {
      let signature = build_signature(args?, return_type?, total?)?;
      Ok(get_corvus_class!("Signature").wrap_data(signature, &*WRAPPER))
    }).unwrap_or_else(raise_and_return_nil)
  }
);

pub fn build_signature(
  args: Array,
  return_type: CorvusType,
  total: Boolean,
) -> ::ruru::result::Result<Signature> {
  let mut signature = Signature::with_capacity(args.length() as usize);

  signature.set_return_type(return_type.clone_type());
  signature.set_total(total.to_bool());

  for arg in args {
    let hash: Hash = arg.try_convert_to()?;
    let name: RString = hash.at(&*SYM_NAME).try_convert_to()?;
    let ty: CorvusType = hash.at(&*SYM_TYPE).try_convert_to()?;
    signature.add_argument(Argument {
      name: name.to_string(),
      ty: ty.clone_type(),
      required: truthy(hash.at(&*SYM_OPTIONAL)),
      variadic: truthy(hash.at(&*SYM_VARIADIC)),
    })
  }
  Ok(signature)
}

pub fn init() {
  init_corvus_class!("Signature", |class| {
    class.def_self("new", corvus_signature_new);
  });
}
