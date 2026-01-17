# GitHub Actions Workflows

This directory contains the CI/CD workflows for AgTerm.

## Workflows

### CI Pipeline (`ci.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main`
- Weekly scheduled runs (Sundays at midnight UTC)

**Jobs:**

1. **Code Quality Checks**
   - Formatting check (`cargo fmt`)
   - Linting with Clippy (all warnings as errors)
   - Documentation build and validation
   - Generates detailed reports in GitHub Step Summary

2. **Test Suite**
   - Runs on Linux, macOS, and Windows
   - Tests against stable and beta Rust channels
   - Comprehensive test output with reports
   - Uploads test results as artifacts

3. **Security Audit**
   - `cargo-audit` for vulnerability scanning
   - `cargo-deny` for license and dependency policy checks

4. **Code Coverage**
   - Generates coverage reports using `cargo-llvm-cov`
   - HTML coverage report uploaded as artifact
   - Uploads to Codecov
   - Summary in GitHub Step Summary

5. **Performance Benchmarks**
   - Runs benchmarks (if available)
   - Tracks performance over time
   - Auto-commits results to gh-pages

6. **Build Release**
   - Multi-platform release builds (Linux, macOS, Windows)
   - Cross-compilation for ARM64
   - Only runs on `main` branch pushes
   - Uploads build artifacts

7. **Dependabot Auto-merge**
   - Automatically merges minor/patch updates from Dependabot
   - Only after all checks pass

### Release Pipeline (`release.yml`)

**Triggers:**
- Push of tags matching `v*` (e.g., `v1.0.0`)
- Manual workflow dispatch with version input

**Jobs:**

1. **Build**
   - Multi-platform builds (Linux, macOS, Windows)
   - Both AMD64 and ARM64 architectures
   - Runs tests before building
   - Binary smoke tests
   - Strips debug symbols
   - Creates macOS .app bundles
   - Generates archives (tar.gz for Unix, zip for Windows)

2. **Build Linux Packages**
   - Creates DEB packages (for Debian/Ubuntu)
   - Creates RPM packages (for Fedora/RHEL)
   - Both AMD64 and ARM64 architectures

3. **Create Release**
   - Creates GitHub Release with generated notes
   - Uploads all build artifacts
   - Generates SHA256 checksums
   - Comprehensive installation instructions

4. **Update Homebrew**
   - Auto-updates Homebrew formula
   - Only for stable releases (not alpha/beta/rc)

5. **Publish to crates.io**
   - Publishes to crates.io registry
   - Only for stable releases

### Dependency Review (`dependency-review.yml`)

**Triggers:**
- Pull requests
- Weekly on Mondays at 9 AM UTC

**Jobs:**

1. **Dependency Review**
   - Reviews dependency changes in PRs
   - Fails on moderate+ severity vulnerabilities
   - Checks license compliance

2. **Outdated Check**
   - Scans for outdated dependencies
   - Generates detailed report

3. **License Check**
   - Generates license compliance report
   - Exports in JSON and TSV formats

### Auto Labeling (`labeler.yml`)

**Triggers:**
- PR opened/updated
- Issue opened/edited

**Jobs:**

1. **Label PR** - Auto-labels based on changed files
2. **Size Label** - Labels PRs by size (XS/S/M/L/XL)

## Configuration Files

### `deny.toml`
Configuration for `cargo-deny`:
- Security vulnerability policies
- License allowlist/denylist
- Dependency banning rules
- Source repository policies

### `codecov.yml`
Configuration for Codecov:
- Coverage targets (70% project, 80% patch)
- Report formatting
- File exclusions

### `labeler.yml`
Configuration for automatic PR labeling:
- File patterns mapped to labels
- Covers code, tests, docs, CI/CD, etc.

## Secrets Required

The following secrets need to be configured in GitHub repository settings:

| Secret | Purpose | Required For |
|--------|---------|--------------|
| `GITHUB_TOKEN` | GitHub API access | Auto-provided |
| `CODECOV_TOKEN` | Codecov uploads | Coverage reporting |
| `HOMEBREW_TAP_TOKEN` | Homebrew formula updates | Release automation |
| `CRATES_IO_TOKEN` | crates.io publishing | Release automation |

## Badges

Add these badges to your README:

```markdown
[![CI](https://github.com/coldwoong-moon/agterm/workflows/CI/badge.svg)](https://github.com/coldwoong-moon/agterm/actions/workflows/ci.yml)
[![Release](https://github.com/coldwoong-moon/agterm/workflows/Release/badge.svg)](https://github.com/coldwoong-moon/agterm/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/coldwoong-moon/agterm/branch/main/graph/badge.svg)](https://codecov.io/gh/coldwoong-moon/agterm)
[![Security Audit](https://github.com/coldwoong-moon/agterm/workflows/Dependency%20Review/badge.svg)](https://github.com/coldwoong-moon/agterm/actions/workflows/dependency-review.yml)
```

## Development Workflow

### Running Checks Locally

Before pushing, run these commands locally:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Check documentation
cargo doc --no-deps --all-features

# Security audit
cargo install cargo-audit
cargo audit

# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated
```

### Creating a Release

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes: `git commit -m "chore: bump version to X.Y.Z"`
4. Create and push tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. Release workflow will automatically build and publish

### Manual Release Trigger

You can manually trigger a release:

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version number (e.g., `1.0.0`)
4. Click "Run workflow"

## Troubleshooting

### CI Failures

**Formatting Errors:**
```bash
cargo fmt --all
```

**Clippy Warnings:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Test Failures:**
```bash
cargo test --all-features -- --nocapture
```

**Coverage Issues:**
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features
```

### Release Issues

**Missing Secrets:**
- Check repository settings → Secrets → Actions
- Verify all required secrets are configured

**Build Failures:**
- Check the specific platform/architecture that failed
- Review build logs for dependency or compilation errors

**Package Publishing Failures:**
- Verify crates.io token is valid
- Check if version already exists
- Ensure Cargo.toml is properly configured

## Performance

### Caching Strategy

We use `Swatinem/rust-cache@v2` for intelligent caching:
- Caches are shared across similar jobs
- Automatic cache key generation based on dependencies
- Faster builds (typically 2-5x speedup after first run)

### Optimization Tips

1. Use matrix strategy for parallel builds
2. Leverage caching aggressively
3. Use `continue-on-error` for non-critical steps
4. Split large workflows into separate jobs

## Maintenance

### Regular Updates

- **Weekly:** Review dependency audit results
- **Monthly:** Update workflow actions to latest versions
- **Quarterly:** Review and optimize CI performance

### Action Version Updates

Check for updates to GitHub Actions:
```bash
# Example for updating checkout action
uses: actions/checkout@v4  # Keep up to date
```

## Contributing

When modifying workflows:

1. Test changes in a fork first
2. Document any new secrets or configuration
3. Update this README with changes
4. Consider backward compatibility
5. Use meaningful commit messages

## Support

For issues with CI/CD:
1. Check workflow run logs
2. Review this documentation
3. Search existing issues
4. Create new issue with `ci-cd` label
