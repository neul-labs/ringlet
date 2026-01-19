require 'fileutils'
require 'net/http'
require 'uri'
require 'tempfile'
require 'rubygems/package'
require 'zlib'

module Clown
  VERSION = '0.1.0'
  GITHUB_REPO = 'neul-labs/ccswitch'

  class << self
    def binary_dir
      File.join(File.dirname(__FILE__), '..', 'native')
    end

    def platform_suffix
      case RUBY_PLATFORM
      when /darwin.*arm64/, /darwin.*aarch64/
        'darwin-arm64'
      when /darwin/
        'darwin-x64'
      when /linux.*arm64/, /linux.*aarch64/
        'linux-arm64'
      when /linux/
        'linux-x64'
      when /mingw|mswin/
        'win32-x64'
      else
        raise "Unsupported platform: #{RUBY_PLATFORM}"
      end
    end

    def binary_extension
      RUBY_PLATFORM =~ /mingw|mswin/ ? '.exe' : ''
    end

    def ensure_binary(name)
      binary_path = File.join(binary_dir, "#{name}#{binary_extension}")

      unless File.exist?(binary_path)
        download_binaries
      end

      binary_path
    end

    def download_binaries
      FileUtils.mkdir_p(binary_dir)

      suffix = platform_suffix
      ext = suffix.start_with?('win') ? 'zip' : 'tar.gz'
      url = "https://github.com/#{GITHUB_REPO}/releases/download/v#{VERSION}/clown-#{suffix}-#{VERSION}.#{ext}"

      warn "Downloading clown v#{VERSION}..."

      uri = URI.parse(url)
      download_with_redirect(uri) do |response|
        Dir.mktmpdir do |tmpdir|
          archive_path = File.join(tmpdir, "clown.#{ext}")
          File.binwrite(archive_path, response.body)

          if ext == 'tar.gz'
            extract_tar_gz(archive_path, binary_dir)
          else
            extract_zip(archive_path, binary_dir)
          end
        end
      end

      # Make executable on Unix
      unless RUBY_PLATFORM =~ /mingw|mswin/
        clown_path = File.join(binary_dir, 'clown')
        clownd_path = File.join(binary_dir, 'clownd')
        File.chmod(0755, clown_path) if File.exist?(clown_path)
        File.chmod(0755, clownd_path) if File.exist?(clownd_path)
      end

      warn "clown binaries installed successfully"
    end

    private

    def download_with_redirect(uri, limit = 5, &block)
      raise 'Too many redirects' if limit == 0

      Net::HTTP.start(uri.host, uri.port, use_ssl: uri.scheme == 'https') do |http|
        request = Net::HTTP::Get.new(uri)
        response = http.request(request)

        case response
        when Net::HTTPSuccess
          block.call(response)
        when Net::HTTPRedirection
          location = response['location']
          new_uri = URI.parse(location)
          download_with_redirect(new_uri, limit - 1, &block)
        else
          raise "Download failed: #{response.code} #{response.message}"
        end
      end
    end

    def extract_tar_gz(archive, dest)
      Zlib::GzipReader.open(archive) do |gz|
        Gem::Package::TarReader.new(gz) do |tar|
          tar.each do |entry|
            next unless entry.file?
            path = File.join(dest, entry.full_name)
            FileUtils.mkdir_p(File.dirname(path))
            File.binwrite(path, entry.read)
          end
        end
      end
    end

    def extract_zip(archive, dest)
      # Simple zip extraction using system command on Windows
      if RUBY_PLATFORM =~ /mingw|mswin/
        system("powershell -Command \"Expand-Archive -Path '#{archive}' -DestinationPath '#{dest}' -Force\"")
      else
        system("unzip -q '#{archive}' -d '#{dest}'")
      end
    end
  end
end
