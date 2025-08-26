class RuneVcs < Formula
  desc "Modern, intelligent version control system"
  homepage "https://github.com/Johan-Ott/rune-vcs"
  url "https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-0.3.0-alpha.4-x86_64-apple-darwin.tar.gz"
  sha256 "02a35b8277bcb52872c65b38709ba0aba5bc59dcbadf1a9693c61d2b71a55fd8"
  license "Apache-2.0"
  version "0.3.0-alpha.4"

  on_arm do
    url "https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-0.3.0-alpha.4-aarch64-apple-darwin.tar.gz"
    sha256 "402c9e99320e4452d9ddaa5aff6b8a2134fef31e3649d0cfd3caa807534ef0bd"
  end

  def install
    bin.install "rune" => "rune-vcs"
  end

  test do
    system "#{bin}/rune-vcs", "--version"
  end
end
