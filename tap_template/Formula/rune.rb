
# Create a repo like CaptainOtto/homebrew-tap and place this file there.
class Rune < Formula
  desc "Rune — modern DVCS"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  version "v0.0.2"
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.2/rune-v0.0.2-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    else
      url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.2/rune-v0.0.2-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    end
  end
  on_linux do
    url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.2/rune-v0.0.2-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_ME"
  endke CaptainOtto/homebrew-tap and place this file there.
class Rune < Formula
  desc "Rune — modern DVCS"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  version "v0.0.1"
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.1/rune-v0.0.1-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    else
      url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.1/rune-v0.0.1-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME"
    end
  end
  on_linux do
    url "https://github.com/CaptainOtto/rune-vcs/releases/download/v0.0.1/rune-v0.0.1-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_ME"
  end
  def install
    bin.install "rune"
    bash_completion.install "rune.bash" => "rune"
    zsh_completion.install "rune.zsh" => "_rune"
    fish_completion.install "rune.fish"
  end
end
