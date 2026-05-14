class Minnal < Formula
  desc "Possibly the greatest coding agent ever built — blazing-fast TUI, multi-model"
  homepage "https://github.com/iamrohithrnair/minnal"
  version "0.14.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/iamrohithrnair/minnal/releases/download/v0.14.0/minnal-macos-aarch64.tar.gz"
      sha256 "REPLACE_WITH_MACOS_AARCH64_SHA256"

      def install
        bin.install "minnal-macos-aarch64" => "minnal"
      end
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/iamrohithrnair/minnal/releases/download/v0.14.0/minnal-linux-x86_64.tar.gz"
      sha256 "REPLACE_WITH_LINUX_X86_64_SHA256"

      def install
        libexec.install "minnal-linux-x86_64", "minnal-linux-x86_64.bin"
        libexec.install Dir["libssl.so*"], Dir["libcrypto.so*"]
        (bin/"minnal").write <<~SH
          #!/bin/sh
          exec "#{libexec}/minnal-linux-x86_64" "$@"
        SH
      end
    end

    on_arm do
      url "https://github.com/iamrohithrnair/minnal/releases/download/v0.14.0/minnal-linux-aarch64.tar.gz"
      sha256 "REPLACE_WITH_LINUX_AARCH64_SHA256"

      def install
        bin.install "minnal-linux-aarch64" => "minnal"
      end
    end
  end

  test do
    assert_match "minnal", shell_output("#{bin}/minnal --version")
  end
end
