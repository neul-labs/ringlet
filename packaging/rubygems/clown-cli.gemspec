Gem::Specification.new do |s|
  s.name        = 'clown-cli'
  s.version     = '0.1.0'
  s.summary     = 'CLI orchestrator for coding agents'
  s.description = 'clown is a cross-platform orchestrator for CLI-based coding agents'
  s.authors     = ['Dipankar Sarkar']
  s.homepage    = 'https://github.com/neul-labs/ccswitch'
  s.license     = 'MIT'

  s.files       = Dir['lib/**/*', 'bin/*', 'README.md']
  s.executables = ['clown', 'clownd']
  s.require_paths = ['lib']

  s.required_ruby_version = '>= 2.7.0'

  s.metadata = {
    'homepage_uri' => s.homepage,
    'source_code_uri' => 'https://github.com/neul-labs/ccswitch',
    'changelog_uri' => 'https://github.com/neul-labs/ccswitch/releases'
  }

  s.post_install_message = <<~MSG
    clown has been installed. Run 'clown --help' to get started.
    The binaries will be downloaded on first use.
  MSG
end
