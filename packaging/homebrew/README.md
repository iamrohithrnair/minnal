# Homebrew tap formula for `minnal`

This directory holds the source-of-truth Homebrew formula. The published copy
lives in the tap repository **`iamrohithrnair/homebrew-minnal`** under
`Formula/minnal.rb`. Users install with:

```bash
brew tap iamrohithrnair/minnal
brew install minnal
```

## Cutting a release

1. Bump `version` in `Cargo.toml` (and the matching `minnal` entry in
   `Cargo.lock`) and publish a GitHub release tagged `vX.Y.Z` on
   `iamrohithrnair/minnal`.
2. Build release binaries for each supported Homebrew target:
   ```bash
   rustup target add aarch64-apple-darwin
   cargo build --release --target aarch64-apple-darwin --bin minnal
   ```
3. Package and hash:
   ```bash
   mkdir -p dist
   tar -czf dist/minnal-macos-aarch64.tar.gz -C target/aarch64-apple-darwin/release minnal
   shasum -a 256 dist/*.tar.gz
   ```
4. Upload the tarballs to the GitHub release:
   ```bash
   gh release upload vX.Y.Z dist/minnal-*.tar.gz --repo iamrohithrnair/minnal
   ```
5. Update `version` and the `sha256` placeholders in `minnal.rb`, then copy
   the file to `iamrohithrnair/homebrew-minnal` at `Formula/minnal.rb`, commit, and
   push.
6. Verify:
   ```bash
   brew install --verbose iamrohithrnair/minnal/minnal
   ```
