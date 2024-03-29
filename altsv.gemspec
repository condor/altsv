# frozen_string_literal: true

lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "altsv/version"

Gem::Specification.new do |spec|
  spec.name          = "altsv"
  spec.version       = Altsv::VERSION
  spec.authors       = ["condor"]
  spec.email         = ["condor1226@gmail.com"]

  spec.summary       = %q{an ALTernative ltSV parser / dumper library}
  spec.description   = %q{A more lightweight LTSV handler library partially written in Rust.}
  spec.homepage      = "https://github.com/condor/altsv"

  # Prevent pushing this gem to RubyGems.org. To allow pushes either set the 'allowed_push_host'
  # to allow pushing to a single host or delete this section to allow pushing to any host.
  if spec.respond_to?(:metadata)
    spec.metadata["homepage_uri"] = spec.homepage
    spec.metadata["source_code_uri"] = "https://github.com/condor/altsv"
    spec.metadata["changelog_uri"] = "https://github.com/condor/altsv/blob/master/CHANGELOG.md"
  else
    raise "RubyGems 2.0 or newer is required to protect against " \
      "public gem pushes."
  end

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files         = Dir.chdir(File.expand_path('..', __FILE__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions    = %w(ext/altsv/extconf.rb)

  spec.add_dependency 'helix_runtime', '~> 0.7.5'

  spec.add_development_dependency 'pry'
  spec.add_development_dependency "bundler", "~> 1.17"
  spec.add_development_dependency "rake", "~> 10.0"
  spec.add_development_dependency "rspec", "~> 3.8", '< 4.0'
  spec.add_development_dependency 'ltsv'
end
