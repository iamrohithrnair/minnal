# Homebrew tap release — remaining steps for v0.12.2

Status of work already done (committed on `fix/copilot-auth` as `2b15260e`):

- [x] Bumped `Cargo.toml` and `Cargo.lock` from `0.12.0` → `0.12.2`
- [x] Added `packaging/homebrew/minnal.rb` (formula scaffold, version `0.12.2`)
- [x] Added `packaging/homebrew/README.md` (release runbook)
- [x] Published GitHub release `v0.12.2` on `codeslord/minnal` (no assets attached yet)

---

## What is left to do

### 1. Build the macOS release binaries
```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin

cargo build --release --target aarch64-apple-darwin --bin minnal
cargo build --release --target x86_64-apple-darwin  --bin minnal
```

### 2. Package as tarballs and capture SHA-256s
```bash
mkdir -p dist
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  tar -czf "dist/minnal-${t}.tar.gz" -C "target/${t}/release" minnal
done
shasum -a 256 dist/*.tar.gz   # save these two hashes — needed in step 5
```

### 3. Upload the tarballs to the v0.12.2 release
```bash
gh release upload v0.12.2 dist/minnal-*.tar.gz --repo codeslord/minnal
```

### 4. Create the tap repository
- On GitHub, create a **public** repo named exactly **`homebrew-minnal`** under `codeslord`.
  (Homebrew requires the `homebrew-` prefix; users will tap it as `codeslord/minnal`.)
- Clone it locally:
  ```bash
  git clone https://github.com/codeslord/homebrew-minnal
  cd homebrew-minnal
  mkdir -p Formula
  ```

### 5. Add the formula to the tap
- Copy `packaging/homebrew/minnal.rb` from this repo to `Formula/minnal.rb` in the tap.
- Replace the two placeholders with the SHA-256 values from step 2:
  - `REPLACE_WITH_AARCH64_DARWIN_SHA256`
  - `REPLACE_WITH_X86_64_DARWIN_SHA256`
- Commit and push:
  ```bash
  git add Formula/minnal.rb
  git commit -m "minnal 0.12.2"
  git push
  ```

### 6. Verify the install end-to-end
```bash
brew tap codeslord/minnal
brew install minnal
minnal --version    # should print 0.12.2
```
Optional sanity check:
```bash
brew audit --strict --new-formula codeslord/minnal/minnal
```

---

## Caveats / follow-ups

- **Tag does not include the version bump.** The `v0.12.2` tag points at
  `50d2c68b` on `master`, which still has `Cargo.toml` version `0.12.0`. Any
  binary built from that exact commit will report `0.12.0` and the formula's
  `minnal --version` test will fail Homebrew audit.
  Options:
  1. Merge `fix/copilot-auth` to `master`, then **delete and retag** `v0.12.2`
     at the new master tip before building binaries:
     ```bash
     git tag -d v0.12.2
     git push --delete origin v0.12.2
     git tag v0.12.2 master
     git push origin v0.12.2
     ```
  2. Or cut a fresh release `v0.12.3` from master after merging, and use that
     tag in the formula instead.
- **Linux support** is not in the scaffold. If you want
  `brew install` to work on Linux too, build
  `x86_64-unknown-linux-gnu` (typically via CI/Docker on Mac), upload the
  tarball, and add an `on_linux do … end` block to the formula.
- **License field** in `minnal.rb` is set to `MIT` — confirm against the
  repository `LICENSE` file and adjust if different.
- **Future releases**: just repeat steps 1–5 with the new version and new
  SHA-256s; no need to recreate the tap repo.
