// pub mod corvus_compiler;
#[macro_use]
mod macros;
pub mod corvus_type;
pub mod corvus_signature;
pub mod corvus_namespace;
pub mod corvus_args;
pub mod corvus_compiler;
pub mod corvus_script;


pub fn init() {
  corvus_type::init();
  corvus_signature::init();
  corvus_namespace::init();
  corvus_args::init();
  corvus_compiler::init();
  corvus_script::init();
}
