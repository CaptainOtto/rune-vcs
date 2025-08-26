# Homebrew Formula for Rune VCS
# Place this in your homebrew-tap repository at Formula/rune.rb

class Rune < Formula
  desc "Rune — modern DVCS with virtual workspaces and draft commits"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  url "https://github.com/CaptainOtto/rune-vcs/archive/refs/tags/v0.3.0-alpha.2.tar.gz"
  sha256 "a045ddc24d4a3166adffb159e9ba6022e723f92293f9e0f3939727c5df810d0a"
  version "0.3.0-alpha.2"
  head "https://github.com/CaptainOtto/rune-vcs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/rune-cli")
    
    # Generate completions if available
    generate_completions_from_executable(bin/"rune", "completion")
  end

  test do
    system "#{bin}/rune", "--version"
    system "#{bin}/rune", "--help"
    
    # Test basic functionality
    system "#{bin}/rune", "init"
    assert_predicate testpath/".rune", :exist?
  end
end
