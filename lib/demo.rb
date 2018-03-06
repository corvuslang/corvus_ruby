require 'pry'
require_relative 'corvus'

nachtkrapp = Corvus::Compiler.new

nachtkrapp.types.define \
  'User',
  id: :number,
  name: :string,
  email: :string

nachtkrapp.define do |f|
  f.arg 'add1', :number
  f.returns :number
  f.callback do |args|
    args['add1'] + 1
  end
end
