require 'corvus/version'
require 'corvus/compiler'
require 'corvus/function_builder'
require 'corvus/type_registry'

require 'thermite/fiddle'

File.dirname(File.dirname(__FILE__)).tap do |toplevel_dir|
  Thermite::Fiddle.load_module('initialize_corvus',
                               cargo_project_path: toplevel_dir,
                               ruby_project_path: toplevel_dir)
end
