require "bundler/gem_tasks"
require "rake/testtask"

require 'thermite/tasks'

Thermite::Tasks.new

desc 'Run Rust & Ruby testsuites'
task test: ['thermite:build', 'thermite:test'] do
  # …
end

Rake::TestTask.new(:test) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.test_files = FileList["test/**/*_test.rb"]
end

Rake::TestTask.new(:bench) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.test_files = FileList["test/**/*_bench.rb"]
end

task build: %w[thermite:build]

task :default => :test