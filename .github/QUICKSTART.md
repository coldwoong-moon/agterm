# CI/CD Quick Start Guide

## For Contributors

### Before Pushing Code

Run these commands locally to catch issues early:

```bash
# 1. Format your code
cargo fmt

# 2. Run linter
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --all-features

# 4. Build documentation
cargo doc --no-deps --all-features

# 5. (Optional) Check coverage
cargo install cargo-llvm-cov
cargo llvm-cov --all-features
```

### Creating a Pull Request

1. Push your branch
2. Create PR using the template
3. Wait for CI to pass (usually 10-15 minutes)
4. Address any CI failures
5. Request review

### CI Checks Explained

Your PR will run these checks:

- ✅ **Formatting** - Code must be formatted with `cargo fmt`
- ✅ **Linting** - No clippy warnings allowed
- ✅ **Tests** - All tests must pass on all platforms
- ✅ **Security** - No known vulnerabilities in dependencies
- ✅ **Documentation** - Docs must build without warnings
- ✅ **Coverage** - Test coverage report generated

## For Maintainers

### Creating a Release

#### 1. Prepare Release

```bash
# Update version in Cargo.toml
vim Cargo.toml

# Update CHANGELOG.md
vim CHANGELOG.md

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

#### 2. Create Tag

```bash
# Create and push tag
git tag vX.Y.Z
git push origin vX.Y.Z
```

#### 3. Monitor Release

1. Go to [Actions](../../actions)
2. Watch "Release" workflow
3. Verify all builds succeed
4. Check GitHub Release is created

#### 4. Publish (Automatic)

The following happen automatically:
- ✅ Binaries built for all platforms
- ✅ DEB/RPM packages created
- ✅ GitHub Release created with notes
- ✅ Homebrew formula updated (if configured)
- ✅ Published to crates.io (if configured)

### Manual Release Trigger

If needed, manually trigger a release:

1. Go to [Actions → Release](../../actions/workflows/release.yml)
2. Click "Run workflow"
3. Enter version (e.g., `1.0.0`)
4. Click "Run workflow"

### Emergency Fixes

For urgent security fixes:

```bash
# 1. Create fix
git checkout -b hotfix/security-issue

# 2. Make changes and test
cargo test --all-features

# 3. Create PR (will run all checks)
git push origin hotfix/security-issue

# 4. After merge, immediately release
git checkout main
git pull
git tag vX.Y.Z
git push origin vX.Y.Z
```

## Troubleshooting

### CI Failed - Formatting

```bash
# Fix formatting locally
cargo fmt

# Commit and push
git add .
git commit -m "fix: formatting"
git push
```

### CI Failed - Clippy

```bash
# Check what's wrong
cargo clippy --all-targets --all-features -- -D warnings

# Fix issues and push
git add .
git commit -m "fix: clippy warnings"
git push
```

### CI Failed - Tests

```bash
# Run tests locally with output
cargo test --all-features -- --nocapture

# Fix failing tests
# Commit and push
git add .
git commit -m "fix: failing tests"
git push
```

### CI Failed - Security Audit

```bash
# Check vulnerabilities
cargo audit

# Update dependencies
cargo update

# If vulnerability has no fix, check if it affects your code
# May need to wait for upstream fix or use different dependency
```

### Release Failed

1. Check [Release workflow logs](../../actions/workflows/release.yml)
2. Common issues:
   - Missing secrets (HOMEBREW_TAP_TOKEN, CRATES_IO_TOKEN)
   - Version already exists on crates.io
   - Test failures

### Coverage Report Not Generated

Coverage reports are optional and won't fail CI. If needed:

```bash
# Generate locally
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --html

# Open report
open target/llvm-cov/html/index.html
```

## Monitoring

### Check Workflow Status

- [CI Workflow](../../actions/workflows/ci.yml)
- [Release Workflow](../../actions/workflows/release.yml)
- [Dependency Review](../../actions/workflows/dependency-review.yml)

### View Reports

- **Test Results:** Check workflow artifacts
- **Coverage:** Check Codecov (if configured)
- **Security Audits:** Check dependency-review workflow

## Configuration

### Required Secrets

Only if you want these features:

| Secret | For | Setup |
|--------|-----|-------|
| `CODECOV_TOKEN` | Coverage reports | [codecov.io](https://codecov.io) |
| `HOMEBREW_TAP_TOKEN` | Homebrew updates | GitHub PAT with repo access |
| `CRATES_IO_TOKEN` | crates.io publish | [crates.io/settings/tokens](https://crates.io/settings/tokens) |

### Optional Features

Some features work without configuration:
- ✅ All CI checks
- ✅ GitHub Releases
- ✅ Binary artifacts
- ✅ Security audits

Features requiring setup:
- ⚙️ Codecov integration (optional)
- ⚙️ Homebrew publishing (optional)
- ⚙️ crates.io publishing (optional)

## Tips

### Speed Up CI

1. **Cache is your friend** - First run is slow, subsequent runs are fast
2. **Test locally first** - Catch issues before pushing
3. **Small PRs** - Easier to review and debug

### Best Practices

1. **Atomic commits** - One logical change per commit
2. **Clear messages** - Descriptive commit messages
3. **Test coverage** - Add tests for new features
4. **Documentation** - Update docs with code changes

### Common Commands

```bash
# Full local check (mimics CI)
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features

# Quick check
cargo check --all-features

# Watch mode for development
cargo watch -x check -x test

# Check specific target
cargo check --target x86_64-unknown-linux-gnu

# Verbose test output
cargo test --all-features -- --nocapture --test-threads=1
```

## Getting Help

1. **Documentation** - Check `.github/workflows/README.md`
2. **Issues** - Search existing issues
3. **Discussions** - Ask in GitHub Discussions
4. **Logs** - Check workflow run logs

## Useful Links

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Rust CI Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [rust-cache](https://github.com/Swatinem/rust-cache)

---

**Questions?** Create an issue with the `ci-cd` label!
