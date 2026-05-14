# Homebrew tap release checklist for v0.13.1

The automated release workflow should publish `v0.13.1` assets and update the
`iamrohithrnair/homebrew-minnal` tap when the `HOMEBREW_DEPLOY_KEY` repository secret
is configured.

Manual fallback if the tap update is skipped or fails:

1. Ensure the public tap repo exists:
   ```bash
   gh repo create iamrohithrnair/homebrew-minnal --public --description "Homebrew tap for minnal"
   ```
2. Wait for the `v0.13.1` release assets to finish uploading.
3. Compute SHA-256 hashes for:
   - `minnal-linux-x86_64.tar.gz`
   - `minnal-linux-aarch64.tar.gz`
   - `minnal-macos-aarch64.tar.gz`
4. Replace the placeholders in `packaging/homebrew/minnal.rb`, copy it to
   `Formula/minnal.rb` in `iamrohithrnair/homebrew-minnal`, commit, and push.
5. Verify:
   ```bash
   brew update
   brew tap iamrohithrnair/minnal
   brew install minnal
   minnal --version
   ```
