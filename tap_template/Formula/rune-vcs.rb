class RuneVcs < Formula
  desc "Modern, intelligent version control system"
  homepage "https://github.com/Johan-Ott/rune-vcs"
  url "https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-v0.3.0-alpha.4-x86_64-apple-darwin.tar.gz"
  sha256 "087560cb92af12057c0dc7a653adf2d79f8c48f0b32b1151eea4b166410da0b2"
  license "Apache-2.0"
  version "0.3.0-alpha.4"

  on_arm do
    url "https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-v0.3.0-alpha.4-aarch64-apple-darwin.tar.gz"
    sha256 "cc0e034e549d7fcf4e9bfa64a6b4ca83e4e49dc82cca199677210ea77ece0aaa"
  end

  def install
    bin.install "rune" => "rune-vcs"
  end

  test do
    system "#{bin}/rune-vcs", "--version"
  end
end
