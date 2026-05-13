# Homebrew tap release checklist for v0.13.0

The automated release workflow should publish `v0.13.0` assets and update the
`codeslord/homebrew-minnal` tap when the `HOMEBREW_DEPLOY_KEY` repository secret
is configured.

Manual fallback if the tap update is skipped or fails:

1. Ensure the public tap repo exists:
   ```bash
   gh repo create codeslord/homebrew-minnal --public --description "Homebrew tap for minnal"
   ```
2. Wait for the `v0.13.0` release assets to finish uploading.
3. Compute SHA-256 hashes for:
   - `minnal-linux-x86_64.tar.gz`
   - `minnal-linux-aarch64.tar.gz`
   - `minnal-macos-aarch64.tar.gz`
   - `minnal-macos-x86_64.tar.gz`
4. Replace the placeholders in `packaging/homebrew/minnal.rb`, copy it to
   `Formula/minnal.rb` in `codeslord/homebrew-minnal`, commit, and push.
5. Verify:
   ```bash
   brew update
   brew tap codeslord/minnal
   brew install minnal
   minnal --version
   ```
