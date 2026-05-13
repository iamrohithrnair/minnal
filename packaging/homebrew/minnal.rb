class Minnal < Formula
  desc "Possibly the greatest coding agent ever built — blazing-fast TUI, multi-model"
  homepage "https://github.com/codeslord/minnal"
  version "0.12.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/codeslord/minnal/releases/download/v0.12.2/minnal-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_AARCH64_DARWIN_SHA256"
    end
    on_intel do
      url "https://github.com/codeslord/minnal/releases/download/v0.12.2/minnal-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_DARWIN_SHA256"
    end
  end

  def install
    bin.install "minnal"
  end

  test do
    assert_match "minnal", shell_output("#{bin}/minnal --version")
  end
end
