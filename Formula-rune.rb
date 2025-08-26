class Rune < Formula
  desc "Rune VCS - Modern, scalable version control system"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.2.0/rune-v0.2.0-aarch64-apple-darwin.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  version "0.2.0"

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
