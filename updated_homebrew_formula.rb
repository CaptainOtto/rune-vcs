class RuneVcs < Formula
  desc "Rune VCS - Modern, scalable version control system with virtual workspaces and draft commits"
  homepage "https://github.com/CaptainOtto/rune-vcs"
  url "https://github.com/CaptainOtto/rune-vcs/archive/refs/tags/v0.3.0-alpha.2.tar.gz"
  sha256 "a045ddc24d4a3166adffb159e9ba6022e723f92293f9e0f3939727c5df810d0a"
  license "Apache-2.0"
  version "0.3.0-alpha.2"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"rune", "--version"
    assert_match "rune #{version}", shell_output("#{bin}/rune --version")
  end
end
