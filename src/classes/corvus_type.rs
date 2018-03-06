use ruru::{AnyObject, Array, Class, Hash, Object, RString, Symbol};
use corvus_core::{RecordField, Type};
use helpers::raise_and_return_nil;

lazy_static!(
  static ref SYM_TYPE: Symbol = Symbol::new("type");
  static ref SYM_OPTIONAL: Symbol = Symbol::new("required");
  static ref SYM_INPUTS: Symbol = Symbol::new("inputs");
  static ref SYM_OUTPUT: Symbol = Symbol::new("output");
);

wrappable_struct!(Type, TypeWrapper, WRAPPER);

class!(CorvusType);
verify_with_class_name!(CorvusType, "Type");
methods!(
    CorvusType,
    itself,

    fn ruby_type_to_s() -> RString {
        let ty: &Type = itself.get_data(&*WRAPPER);
        RString::new(format!("{}", ty).as_str())
    }

    fn ruby_type_inspect() -> RString {
        let ty: &Type = itself.get_data(&*WRAPPER);
        RString::new(format!("<Corvus::Type \"{}\">", ty).as_str())
    }

    fn ruby_make_record_type(fields: Hash) -> AnyObject {
        use std::collections::HashMap;
        use ruru::result::Error;

        fields.and_then(|fields| {
            let mut field_types: HashMap<String, RecordField> = HashMap::new();
            let mut conversion_errors: Vec<Error> = vec![];

            fields.each(|key, value| {
                use helpers::stringify_key;

                stringify_key(key).and_then(|field_name| {
                    value.try_convert_to::<Hash>().map(|spec| {
                        let field_ty = spec.at(&*SYM_TYPE).get_data(&*WRAPPER).clone();
                        let optional = spec.at(&*SYM_OPTIONAL).value().is_true();
                        field_types.insert(field_name.to_string(), RecordField::new(field_ty, optional));
                    })
                }).unwrap_or_else(|err| {
                    conversion_errors.push(err);
                })
            });

            if conversion_errors.is_empty() {
                Ok(get_corvus_class!("Type").wrap_data(Type::Record(true, field_types), &*WRAPPER))
            } else {
                Err(conversion_errors.remove(0))
            }
        }).unwrap_or_else(raise_and_return_nil)
    }

    fn ruby_make_type_var(name: RString) -> AnyObject {
        name.map(|name| {
            get_corvus_class!("Type").wrap_data(Type::var(name.to_string().as_str()), &*WRAPPER)
        }).unwrap_or_else(raise_and_return_nil)
    }

    fn ruby_make_list_type(ty: CorvusType) -> AnyObject {
        ty.map(|ty| {
            let inner_ty: &Type = ty.get_data(&*WRAPPER);
            get_corvus_class!("Type").wrap_data(Type::list_of(inner_ty.clone()), &*WRAPPER)
        }).unwrap_or_else(raise_and_return_nil)
    }

    fn ruby_make_block_type(params: Hash) -> AnyObject {
        params.and_then(|params| {
            params.at(&*SYM_INPUTS)
                .try_convert_to::<Array>()
                .map(|inputs: Array| {
                    inputs.into_iter().map(|element| element.get_data(&*WRAPPER).clone()).collect()
                }).map(|input_types: Vec<Type>| {
                    let output_type = params.at(&*SYM_OUTPUT).get_data(&*WRAPPER).clone();
                    let block_type = Type::Block(input_types, Box::new(output_type));
                    get_corvus_class!("Type").wrap_data(block_type, &*WRAPPER)
                })
        }).unwrap_or_else(raise_and_return_nil)
    }
);

impl CorvusType {
    pub fn new(ty: Type) -> AnyObject {
        get_corvus_class!("Type").wrap_data(ty, &*WRAPPER)
    }

    pub fn clone_type(&self) -> Type {
        self.get_data(&*WRAPPER).clone()
    }
}

pub fn init() {
    init_corvus_class!("Type", |class| {
        // type constructors
        class.def_self("record", ruby_make_record_type);
        class.def_self("var", ruby_make_type_var);
        class.def_self("block", ruby_make_block_type);
        class.def_self("list", ruby_make_list_type);

        // Primitive type constants
        for (const_name, ty) in vec![
            ("Any", Type::Any),
            ("Number", Type::Num),
            ("String", Type::Str),
            ("Bool", Type::Bool),
            ("Time", Type::Time),
        ] {
            let t: AnyObject = get_corvus_class!("Type").wrap_data(ty, &*WRAPPER);
            class.const_set(const_name, &t);
        }


        class.def("to_s", ruby_type_to_s);
        class.def("inspect", ruby_type_inspect);
    });
}
