
module Corvus
  class Compiler
    attr_reader :types

    def initialize
      @ns = Namespace.new
      @types = TypeRegistry.new
    end

    def define
      builder = FunctionBuilder.new(@types)
      yield builder
      @ns.define(*builder.into_parts)
    end

    def corvus_call(*args)
      @ns.corvus_call(*args)
    end

    # other methods defined in Rust:
    #
    # def compile(corvus_source_code) => CorvusScript
    #
  end
end
