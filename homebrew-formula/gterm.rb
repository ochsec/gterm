class Gterm < Formula
  desc "Flyweight IDE in your terminal - TUI code editor inspired by Geany"
  homepage "https://github.com/ochsec/gterm"
  version "0.3.0"
  license "GPL-2.0"

  on_macos do
    on_arm do
      url "https://github.com/ochsec/gterm/releases/download/v#{version}/gterm-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_intel do
      url "https://github.com/ochsec/gterm/releases/download/v#{version}/gterm-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/ochsec/gterm/releases/download/v#{version}/gterm-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_intel do
      url "https://github.com/ochsec/gterm/releases/download/v#{version}/gterm-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "gterm"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/gterm --version 2>&1", 1)
  end
end
