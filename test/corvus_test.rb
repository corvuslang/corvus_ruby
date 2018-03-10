require "test_helper"

class CorvusTest < Minitest::Test
  def setup
    @compiler = Corvus::Compiler.new
  end

  def test_that_it_has_a_version_number
    refute_nil ::Corvus::VERSION
  end

  def test_it_can_compile_and_call_scripts
    script = @compiler.compile 'calc: 1 plus: 3'
    assert_equal script.call, 4.0
  end

  def test_it_can_define_and_resolve_types
    @compiler.types.define \
      'Person',
      name: :string,
      age: :number,
      favourite_dessert: :string

    @compiler.types.define \
      'Location',
      name: :string,
      street_address: :string,
      lat: :number,
      lon: :number

    @compiler.types.define \
      'Salon',
      location: 'Location',
      people: @compiler.types.list_of('Person')

    assert_equal \
      Corvus::Type::Number,
      @compiler.types.resolve('Salon').fields['location'].fields['lat']
  end

  def test_it_can_infer_required_input_types
    script = @compiler.compile 'calc: x plus: y'
    script.input_types
  end
end
