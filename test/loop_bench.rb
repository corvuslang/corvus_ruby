require 'test_helper'
require 'minitest/benchmark'

class NestedLoopBenchmark < Minitest::Benchmark
  def self.bench_range
    [10, 100, 1000]
  end

  def setup
    @bird = Corvus::Compiler.new

    @script = @bird.compile '
      each: { countFrom: 1 to: n } do: { i =>
        each: { countFrom: 1 to: i } do: { j => stringify: calc: i times: j }
      }
    '.strip

    @pure_ruby = Proc.new do |n|
      (1..n).map do |i|
        (1..i).map do |j|
          (i * j).to_s
        end
      end
    end
  end

  def bench_pure_ruby
    assert_performance_linear do |n|
      @pure_ruby.call(n)
    end
  end

  def bench_corvus_compiled
    assert_performance_linear do |n|
      @script.call(n: n.to_f)
    end
  end

  def bench_corvus_interpreted
    assert_performance_linear do |n|
      @script.call_interpreted(n: n.to_f)
    end
  end
end
