macro_rules! verify_with_class_name {
  ($rust_name:ident, $ruby_name:expr) => {
    impl ::ruru::VerifiedObject for $rust_name {
      fn is_correct_type<O: ::ruru::Object>(object: &O) -> bool {
        object.class() == get_corvus_class!($ruby_name)
      }

      fn error_message() -> &'static str {
        concat!("object is not a Corvus::", $ruby_name)
      }
    }
  };
}

macro_rules! attr_reader {
  ($class:ident, $type_name:ident, $fn_name:ident, $var_name:ident) => {
    pub extern "C" fn $fn_name(
      _argc: ::ruru::types::Argc,
      _argv: *const ::ruru::AnyObject,
      itself: $type_name,
    ) -> ::ruru::AnyObject {
      itself.instance_variable_get(concat!("@", stringify!(var_name)))
    }

    $class.def(stringify!(var_name), $fn_name);
  };
}

macro_rules! init_corvus_class {
  ($class_name:expr, $closure:expr) => {
    Class::from_existing("Corvus").define(|module| {
      let data = Class::from_existing("Data");
      module.define_nested_class($class_name, Some(&data)).define($closure);
    })
  }
}

macro_rules! get_corvus_class {
  ($class_name:expr) => {
    Class::from_existing("Corvus").get_nested_class($class_name)
  }
}
