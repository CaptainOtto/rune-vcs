class Rune < Formula
  desc "Revolutionary AI-powered version control system with natural language interface"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.3.1-alpha.5/rune-v0.3.1-alpha.5-aarch64-apple-darwin.tar.gz"
  sha256 "7f8f000d5e878848e6b6e17605d6a7fda0a5c308b078ca7a58eed8a7c7b2c2ad"
  license "MIT"
  version "0.3.1-alpha.5"

  def install
    bin.install "rune"
  end

  test do
    system "#{bin}/rune", "--version"
  end
end
