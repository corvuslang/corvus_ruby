
module Corvus
  class FunctionBuilder
    def initialize(types)
      @types = types
      @total = false
    end

    def arg(name, type, optional: false, variadic: false)
      @args ||= []
      @args << { name: name,
                 type: @types.resolve(type),
                 optional: optional,
                 variadic: variadic }
    end

    def returns(type)
      @return_type = @types.resolve(type)
    end

    def callback(&block)
      @callback = block
    end

    def partial!
      @total = false
    end

    def total!
      @total = true
    end

    def into_parts
      raise 'Function must have at least one argument' if @args.empty?
      [@args, @return_type, @total, @callback]
    end
  end
end