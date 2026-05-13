# Homebrew tap formula for `minnal`

This directory holds the source-of-truth Homebrew formula. The published copy
lives in the tap repository **`codeslord/homebrew-minnal`** under
`Formula/minnal.rb`. Users install with:

```bash
brew tap codeslord/minnal
brew install minnal
```

## Cutting a release

1. Bump `version` in `Cargo.toml` (and the matching `jcode` entry in
   `Cargo.lock`) and publish a GitHub release tagged `vX.Y.Z` on
   `codeslord/minnal`.
2. Build release binaries for each supported target:
   ```bash
   rustup target add aarch64-apple-darwin x86_64-apple-darwin
   cargo build --release --target aarch64-apple-darwin --bin minnal
   cargo build --release --target x86_64-apple-darwin  --bin minnal
   ```
3. Package and hash:
   ```bash
   mkdir -p dist
   for t in aarch64-apple-darwin x86_64-apple-darwin; do
     tar -czf "dist/minnal-${t}.tar.gz" -C "target/${t}/release" minnal
   done
   shasum -a 256 dist/*.tar.gz
   ```
4. Upload the tarballs to the GitHub release:
   ```bash
   gh release upload vX.Y.Z dist/minnal-*.tar.gz --repo codeslord/minnal
   ```
5. Update `version` and the two `sha256` placeholders in `minnal.rb`, then copy
   the file to `codeslord/homebrew-minnal` at `Formula/minnal.rb`, commit, and
   push.
6. Verify:
   ```bash
   brew install --verbose codeslord/minnal/minnal
   ```
