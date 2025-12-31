# Implementation Plan: Update Dependencies to Latest Versions

## Prerequisites

### Required Tools
- [x] Rust toolchain 1.90.0+ installed
- [x] Cargo available
- [x] Git repository clean state (verified: clean)
- [ ] DynamoDB Local running on localhost:8000

### Required Environment
```bash
# Start DynamoDB Local (required for integration tests)
java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib \
     -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar \
     -inMemory -sharedDb
```

### Pre-flight Checks
```bash
# Verify current state passes all checks
cargo build --all-features          # Should succeed
cargo clippy --all-features -- -D warnings  # Should pass
cargo test --all-features           # Should pass (needs DynamoDB Local)

# Create backup
git stash                           # Stash any uncommitted changes
git checkout -b feature/update-deps # Create feature branch
```

## Implementation Steps

### Phase 1: Preparation and Baseline

#### Step 1.1: Establish Baseline
**Dependencies**: None
**Estimated Time**: 5 minutes

```bash
# Capture current state
cargo build --all-features
cargo clippy --all-features -- -D warnings
cargo test --all-features

# Document current versions
cargo tree --depth 1 > baseline-deps.txt
```

**Verification**:
- [ ] All commands succeed
- [ ] baseline-deps.txt created
- [ ] No uncommitted changes (except baseline file)

---

#### Step 1.2: Review Current Dependency Versions
**Dependencies**: Step 1.1
**Estimated Time**: 5 minutes

```bash
# Check what cargo update would do
cargo update --dry-run

# Search for serde_yaml_ng info
cargo search serde_yaml_ng --limit 1
```

**Verification**:
- [ ] Understand current lock state
- [ ] Confirm serde_yaml_ng v0.10.0 exists

---

### Phase 2: Dependency Updates

#### Step 2.1: Update Cargo.toml - Remove Deprecated serde_yaml
**Dependencies**: Step 1.2
**Estimated Time**: 5 minutes

**Action**: Edit `Cargo.toml` to replace `serde_yaml` with `serde_yaml_ng`

**Changes**:
```toml
# OLD (line 31):
serde_yaml = "0.9"

# NEW:
serde_yaml_ng = "0.10"
```

**Files Modified**:
- `Cargo.toml`

**Verification**:
- [ ] `Cargo.toml` updated
- [ ] No syntax errors in TOML

---

#### Step 2.2: Update Cargo.toml - Cleanup Dev Dependencies
**Dependencies**: Step 2.1
**Estimated Time**: 2 minutes

**Action**: Remove overly-specific version in dev-dependencies

**Changes**:
```toml
# OLD (line 43):
serde_json = "1.0.140"

# NEW:
serde_json = "1"
```

**Files Modified**:
- `Cargo.toml`

**Verification**:
- [ ] `Cargo.toml` updated
- [ ] Consistent versioning style

---

#### Step 2.3: Update src/error.rs - Fix Import
**Dependencies**: Step 2.1
**Estimated Time**: 3 minutes

**Action**: Update error type reference from `serde_yaml` to `serde_yaml_ng`

**Changes**:
```rust
// OLD (line 16):
#[error("Failed to parse configuration file '{0}': {1}")]
ConfigParse(String, #[source] serde_yaml::Error),

// NEW:
#[error("Failed to parse configuration file '{0}': {1}")]
ConfigParse(String, #[source] serde_yaml_ng::Error),
```

**Files Modified**:
- `src/error.rs`

**Verification**:
- [ ] src/error.rs updated
- [ ] No syntax errors

---

#### Step 2.4: Initial Build Check
**Dependencies**: Steps 2.1, 2.2, 2.3
**Estimated Time**: 2 minutes

**Action**: Attempt initial build to identify any immediate issues

```bash
cargo build --all-features
```

**Expected Result**:
- Build should fail initially because `serde_yaml_ng` is not imported in `src/config.rs`

**Verification**:
- [ ] Build attempted
- [ ] Error messages reviewed
- [ ] Root cause identified

---

#### Step 2.5: Update src/config.rs - Add serde_yaml_ng Import
**Dependencies**: Step 2.4
**Estimated Time**: 3 minutes

**Action**: Update import to use `serde_yaml_ng` instead of `serde_yaml`

**Analysis**: Check current usage in `src/config.rs`

```bash
grep -n "serde_yaml" src/config.rs
```

**Changes**: The file currently doesn't have explicit `use serde_yaml`, but uses it implicitly through serde. Need to verify actual usage pattern.

**Investigation Steps**:
1. Read full `src/config.rs` to find YAML usage
2. Identify if `serde_yaml::from_reader` or similar is used
3. Replace with `serde_yaml_ng::from_reader`

**Files Modified**:
- `src/config.rs`

**Verification**:
- [ ] All `serde_yaml` references found
- [ ] Replaced with `serde_yaml_ng`
- [ ] No syntax errors

---

#### Step 2.6: Update Cargo.lock
**Dependencies**: Steps 2.1-2.5
**Estimated Time**: 1 minute

**Action**: Regenerate Cargo.lock with new dependencies

```bash
cargo update
```

**Expected Changes**:
- Remove `serde_yaml` v0.9.34
- Add `serde_yaml_ng` v0.10.0
- Update `serde_json` to latest (if not already)
- Potentially update other transitive dependencies

**Verification**:
- [ ] Cargo.lock updated
- [ ] No version conflicts
- [ ] serde_yaml_ng present, serde_yaml absent

---

### Phase 3: Compilation and Linting

#### Step 3.1: Full Build Verification
**Dependencies**: Step 2.6
**Estimated Time**: 3 minutes

**Action**: Build with all features

```bash
cargo clean
cargo build --all-features
```

**Expected Result**: Successful compilation

**Verification**:
- [ ] Build succeeds
- [ ] No warnings
- [ ] All features compile

**Troubleshooting**: If build fails:
1. Review error messages
2. Check for API incompatibilities
3. Verify all imports updated
4. Check Cargo.toml syntax

---

#### Step 3.2: Check Compilation (Individual Features)
**Dependencies**: Step 3.1
**Estimated Time**: 5 minutes

**Action**: Verify each feature compiles independently

```bash
# Default features
cargo check

# Connector feature only
cargo check --no-default-features --features connector

# Test utils feature
cargo check --no-default-features --features test_utils

# All features
cargo check --all-features
```

**Verification**:
- [ ] Default features compile
- [ ] Connector feature compiles
- [ ] Test utils feature compiles
- [ ] All features compile

---

#### Step 3.3: Run Clippy (All Features)
**Dependencies**: Step 3.1
**Estimated Time**: 3 minutes

**Action**: Run clippy with strict linting

```bash
cargo clippy --all-features -- -D warnings
```

**Expected Result**: No warnings or errors

**Verification**:
- [ ] Clippy passes
- [ ] No new warnings introduced
- [ ] Existing allow rules still valid

**Troubleshooting**: If clippy fails:
1. Review new warnings
2. Assess if legitimate issues or false positives
3. Fix code or update `[lints.clippy]` section
4. Document any new allows with reasoning

---

#### Step 3.4: Run Clippy (Individual Features)
**Dependencies**: Step 3.3
**Estimated Time**: 5 minutes

**Action**: Ensure clippy passes for all feature combinations

```bash
cargo clippy --no-default-features -- -D warnings
cargo clippy --no-default-features --features connector -- -D warnings
cargo clippy --no-default-features --features test_utils -- -D warnings
cargo clippy --all-features -- -D warnings
```

**Verification**:
- [ ] All feature combinations pass clippy
- [ ] No feature-specific warnings

---

### Phase 4: Testing

#### Step 4.1: Unit Tests (if any)
**Dependencies**: Step 3.3
**Estimated Time**: 2 minutes

**Action**: Run any unit tests

```bash
cargo test --lib --all-features
```

**Verification**:
- [ ] Unit tests pass (or no unit tests exist)
- [ ] No test failures

---

#### Step 4.2: Integration Tests - Prerequisites
**Dependencies**: Step 3.3
**Estimated Time**: Variable

**Action**: Ensure DynamoDB Local is running

```bash
# Check if DynamoDB Local is running
curl http://localhost:8000 2>/dev/null || echo "DynamoDB Local not running"

# If not running, start it (see Prerequisites section)
```

**Verification**:
- [ ] DynamoDB Local running on port 8000
- [ ] Port accessible

---

#### Step 4.3: Integration Tests - YAML Loading
**Dependencies**: Steps 4.2
**Estimated Time**: 5 minutes

**Action**: Run tests that exercise YAML parsing functionality

```bash
# Run specific tests that load YAML configs
cargo test --test connector_integration_test --all-features \
    dev_config_should_create_and_describe_table

cargo test --test connector_integration_test --all-features \
    prod_config_should_return_empty_map_without_creating

cargo test --test connector_integration_test --all-features \
    multi_table_config_should_create_all_tables

cargo test --test connector_integration_test --all-features \
    dev_config_should_seed_data
```

**Expected Result**: All YAML-loading tests pass

**Verification**:
- [ ] dev.yml loads correctly
- [ ] prod.yml loads correctly
- [ ] multi_table.yml loads correctly
- [ ] Seed data from JSON loads correctly

**Critical**: These tests verify serde_yaml_ng compatibility

---

#### Step 4.4: Integration Tests - Full Suite
**Dependencies**: Step 4.3
**Estimated Time**: 5 minutes

**Action**: Run complete test suite

```bash
cargo test --all-features
```

**Expected Result**: All tests pass

```
test result: ok. X passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Verification**:
- [ ] All integration tests pass
- [ ] No test failures
- [ ] No ignored tests unexpectedly

**Troubleshooting**: If tests fail:
1. Check DynamoDB Local is running
2. Review test output for specific failures
3. Verify table creation/deletion working
4. Check AWS SDK compatibility
5. Run individual tests for debugging

---

#### Step 4.5: Test with Different Feature Combinations
**Dependencies**: Step 4.4
**Estimated Time**: 5 minutes

**Action**: Verify tests work with different feature sets

```bash
# Tests without test_utils feature (should skip those tests)
cargo test --no-default-features --features connector

# Tests with all features (already done in 4.4)
cargo test --all-features
```

**Verification**:
- [ ] Feature-gated tests properly skipped/included
- [ ] No unexpected test results

---

### Phase 5: Documentation and Verification

#### Step 5.1: Update Documentation References
**Dependencies**: Step 4.4
**Estimated Time**: 5 minutes

**Action**: Check for documentation references to serde_yaml

```bash
# Search for references
grep -r "serde_yaml" . --include="*.md" --include="*.toml" --include="*.rs"

# Update any found references
```

**Files to Check**:
- README.md
- CHANGELOG.md
- Cargo.toml (already updated)
- Source code comments
- .claude/CLAUDE.md

**Verification**:
- [ ] All documentation updated
- [ ] No stale references to serde_yaml
- [ ] serde_yaml_ng mentioned if relevant

---

#### Step 5.2: Generate Updated Dependency Tree
**Dependencies**: Step 5.1
**Estimated Time**: 2 minutes

**Action**: Capture new dependency state

```bash
cargo tree --depth 1 > updated-deps.txt

# Compare with baseline
diff baseline-deps.txt updated-deps.txt
```

**Verification**:
- [ ] updated-deps.txt created
- [ ] Differences documented
- [ ] Expected changes present (serde_yaml → serde_yaml_ng)

---

#### Step 5.3: Final Build and Format Check
**Dependencies**: Step 5.2
**Estimated Time**: 3 minutes

**Action**: Ensure code is properly formatted and builds

```bash
cargo fmt --check
cargo build --all-features
cargo build --release --all-features
```

**Verification**:
- [ ] Code is formatted
- [ ] Debug build succeeds
- [ ] Release build succeeds

---

#### Step 5.4: Pre-commit Hooks Validation
**Dependencies**: Step 5.3
**Estimated Time**: 5 minutes

**Action**: Run pre-commit hooks (if configured)

```bash
pre-commit run --all-files
```

**Expected Checks**:
- cargo fmt
- cargo deny check
- typos
- cargo check
- cargo clippy
- cargo test

**Verification**:
- [ ] All pre-commit hooks pass
- [ ] No failures introduced

---

### Phase 6: Finalization

#### Step 6.1: Review All Changes
**Dependencies**: Step 5.4
**Estimated Time**: 5 minutes

**Action**: Review git diff

```bash
git status
git diff Cargo.toml
git diff src/error.rs
git diff src/config.rs
git diff Cargo.lock | head -100
```

**Verification**:
- [ ] Changes are minimal and focused
- [ ] No unexpected modifications
- [ ] All changes align with plan

---

#### Step 6.2: Commit Changes
**Dependencies**: Step 6.1
**Estimated Time**: 3 minutes

**Action**: Create atomic commits

```bash
# Stage changes
git add Cargo.toml src/error.rs src/config.rs Cargo.lock

# Commit
git commit -m "chore: migrate from deprecated serde_yaml to serde_yaml_ng

- Replace serde_yaml 0.9 with serde_yaml_ng 0.10
- Update error type in src/error.rs
- Update import in src/config.rs
- Update dev-dependency serde_json version specification
- All tests pass, clippy clean

Addresses deprecation warning from serde_yaml crate.
serde_yaml_ng is the official maintained fork."
```

**Verification**:
- [ ] Commit created
- [ ] Commit message descriptive
- [ ] Follows conventional commits format

---

#### Step 6.3: Final Validation
**Dependencies**: Step 6.2
**Estimated Time**: 5 minutes

**Action**: Run complete validation suite one final time

```bash
cargo clean
cargo build --all-features
cargo clippy --all-features -- -D warnings
cargo test --all-features
```

**Verification**:
- [ ] Clean build succeeds
- [ ] Clippy passes
- [ ] All tests pass
- [ ] Ready for PR/merge

---

## Dependencies Between Steps

```
1.1 (Baseline) → 1.2 (Review)
                    ↓
2.1 (Update Cargo.toml) → 2.2 (Dev deps) → 2.3 (Update error.rs) → 2.4 (Build check)
                                                                           ↓
                                                                    2.5 (Update config.rs)
                                                                           ↓
                                                                    2.6 (Cargo update)
                                                                           ↓
3.1 (Full build) → 3.2 (Feature builds) → 3.3 (Clippy all) → 3.4 (Clippy features)
                                                                           ↓
4.1 (Unit tests) → 4.2 (DynamoDB prep) → 4.3 (YAML tests) → 4.4 (Full suite) → 4.5 (Feature tests)
                                                                                          ↓
5.1 (Docs) → 5.2 (Dep tree) → 5.3 (Format) → 5.4 (Pre-commit)
                                                      ↓
6.1 (Review) → 6.2 (Commit) → 6.3 (Final validation)
```

## Verification Checkpoints

### Checkpoint 1: After Phase 2 (Dependency Updates)
- [ ] Cargo.toml updated correctly
- [ ] src/error.rs updated
- [ ] src/config.rs updated
- [ ] Cargo.lock regenerated
- [ ] No version conflicts

### Checkpoint 2: After Phase 3 (Compilation)
- [ ] All feature combinations build
- [ ] Clippy passes with -D warnings
- [ ] No new warnings introduced

### Checkpoint 3: After Phase 4 (Testing)
- [ ] All integration tests pass
- [ ] YAML loading works (critical path)
- [ ] DynamoDB operations functional

### Checkpoint 4: After Phase 5 (Documentation)
- [ ] Documentation updated
- [ ] Code formatted
- [ ] Pre-commit hooks pass

### Checkpoint 5: After Phase 6 (Finalization)
- [ ] Changes committed
- [ ] Final validation complete
- [ ] Ready for merge

## Rollback Procedure

If any checkpoint fails and cannot be resolved:

```bash
# Discard all changes
git checkout Cargo.toml src/error.rs src/config.rs Cargo.lock

# Or reset to last commit
git reset --hard HEAD

# Or reset to specific commit
git reset --hard <commit-hash>

# Update lock file to match Cargo.toml
cargo update
```

## Success Criteria

All of the following must be true:

1. `cargo build --all-features` succeeds
2. `cargo clippy --all-features -- -D warnings` produces no output
3. `cargo test --all-features` shows all tests passing
4. `serde_yaml` removed from Cargo.lock
5. `serde_yaml_ng` present in Cargo.lock at version 0.10.x
6. All YAML fixture files still parse correctly
7. No breaking changes to public API
8. Git history is clean with descriptive commits

## Estimated Total Time

- Phase 1 (Preparation): 10 minutes
- Phase 2 (Updates): 15 minutes
- Phase 3 (Compilation): 15 minutes
- Phase 4 (Testing): 20 minutes
- Phase 5 (Documentation): 15 minutes
- Phase 6 (Finalization): 15 minutes

**Total**: ~90 minutes (including contingency for troubleshooting)

## Notes

- Run tests in a clean environment to avoid state pollution
- Ensure DynamoDB Local is running before integration tests
- Keep baseline-deps.txt and updated-deps.txt for reference
- Document any unexpected issues or discoveries
- Consider running tests multiple times to check for flaky tests
