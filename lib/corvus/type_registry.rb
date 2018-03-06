module Corvus
  class TypeRegistry
    def initialize
      @named_types = {}
    end

    def define(name, type)
      raise "#{name} already defined" if @named_types[name]
      @named_types[name] = resolve(type)
    end

    def resolve(type)
      if type.is_a?(Type) then type
      elsif type.is_a?(Symbol) then resolve_by_name(type.to_s)
      elsif type.is_a?(String) then resolve_by_name(type)
      elsif type.is_a?(Hash) then resolve_record(type)
      else
        "Cannot resolve type from #{type.class.name} #{type}"
      end
    end

    protected

    def resolve_by_name(name)
      case name
      when 'any' then Type::Any
      when 'boolean', 'bool' then Type::Bool
      when 'number', 'num' then Type::Number
      when 'string' then Type::String
      when 'time' then Type::Time
      else
        raise "#{name} is not defined" unless @named_types[name]
        @named_types[name]
      end
    end

    def resolve_record(hash)
      resolved_fields = hash.map do |(key, val)|
        [key.to_s, resolve_field(val)]
      end
      Type.record(resolved_fields.to_h)
    end

    def resolve_field(field)
      if field.is_a?(Symbol) || field.is_a?(String) || field.is_a?(CorvusType)
        { type: resolve(field), optional: false }
      else
        { type: resolve(field[:type]), optional: !!field[:optional] }
      end
    end
  end
end