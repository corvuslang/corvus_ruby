use ruru::{AnyObject, Array, Boolean, Class, Hash, Object, RString, Symbol};
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

    fn corvus_type_self_record(fields: Hash) -> AnyObject {
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

    fn corvus_type_self_var(name: RString) -> AnyObject {
        name.map(|name| {
            get_corvus_class!("Type").wrap_data(Type::var(name.to_string().as_str()), &*WRAPPER)
        }).unwrap_or_else(raise_and_return_nil)
    }

    fn corvus_type_self_list(ty: CorvusType) -> AnyObject {
        ty.map(|ty| {
            let inner_ty: &Type = ty.get_data(&*WRAPPER);
            get_corvus_class!("Type").wrap_data(Type::list_of(inner_ty.clone()), &*WRAPPER)
        }).unwrap_or_else(raise_and_return_nil)
    }

    fn corvus_type_self_block(params: Hash) -> AnyObject {
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

    fn corvus_type_fields() -> Hash {
        let ty: &Type = itself.get_data(&*WRAPPER);
        let mut hash: Hash = Hash::new();
        if let &Type::Record(_partial, ref fields) = ty {
            for (key, field) in fields {
                hash.store(RString::new(key), CorvusType::new(field.get_type().clone()));
            }
        }
        hash
    }

    fn corvus_type_eq(other: AnyObject) -> Boolean {
        Boolean::new(match other.and_then(|ao| ao.try_convert_to::<CorvusType>()) {
            Ok(other) => other.get_data(&*WRAPPER) == itself.get_data(&*WRAPPER),
            _ => false
        })
    }

    fn corvus_type_to_s() -> RString {
        let ty: &Type = itself.get_data(&*WRAPPER);
        RString::new(format!("{}", ty).as_str())
    }

    fn corvus_type_inspect() -> RString {
        let ty: &Type = itself.get_data(&*WRAPPER);
        RString::new(format!("<Corvus::Type \"{}\">", ty).as_str())
    }

    fn corvus_type_check_value(value: AnyObject) -> AnyObject {
        use value::CorvusValue;
        use std::iter::FromIterator;
        value.map(CorvusValue::from).map(|value| {
            let ty: &Type = itself.get_data(&*WRAPPER);
            match ty.satisfied_by_value(&CorvusValue::from(value)) {
                Ok(_) => Array::new().to_any_object(),
                Err(errors) => {
                    Array::from_iter(
                        errors.into_iter().map(|e| RString::from(format!("{}", e)).to_any_object())
                    ).to_any_object()
                }
            }
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
        class.def_self("record", corvus_type_self_record);
        class.def_self("var", corvus_type_self_var);
        class.def_self("block", corvus_type_self_block);
        class.def_self("list", corvus_type_self_list);

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


        class.def("to_s", corvus_type_to_s);
        class.def("==", corvus_type_eq);
        class.def("inspect", corvus_type_inspect);
        class.def("fields", corvus_type_fields);
        class.def("check_value", corvus_type_check_value);
    });
}
