class Rune < Formula
  desc "Rune VCS - Revolutionary AI-powered version control system"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.3.1-alpha.5/rune-v0.3.1-alpha.5-aarch64-apple-darwin.tar.gz"
  sha256 "7f8f000d5e878848e6b6e17605d6a7fda0a5c308b078ca7a58eed8a7c7b2c2ad"
  license "MIT"
  version "0.3.1-alpha.5"

  depends_on "git"

  def install
    bin.install "rune"
    
    # Install shell completions
    bash_completion.install "rune.bash" => "rune"
    zsh_completion.install "rune.zsh" => "_rune"
    fish_completion.install "rune.fish"
  end

  test do
    system "#{bin}/rune", "version"
    
    # Test basic functionality
    system "#{bin}/rune", "help"
  end
end
