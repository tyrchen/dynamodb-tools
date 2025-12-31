# Implementation Plan: Update Dependencies to Latest Versions

## Prerequisites

### Environment Setup

1. **Rust Toolchain**
   - Rust stable toolchain installed (currently using 1.90.0+)
   - cargo-fmt, cargo-clippy available
   - Edition 2024 support

2. **Development Tools**
   - cargo-deny installed for license/security checks
   - cargo-nextest or standard cargo test
   - Git configured for commits

3. **Test Infrastructure**
   - DynamoDB Local running on http://localhost:8000
   - Start with: `java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar -inMemory -sharedDb`
   - Verify accessible: `curl http://localhost:8000`

4. **Repository State**
   - Working directory clean or on feature branch
   - Currently on branch: `chore/update-dependencies` ✓
   - All existing tests passing
   - No uncommitted changes that would conflict

### Verification Commands

```bash
# Verify prerequisites
rustc --version          # Should be 1.90.0 or newer
cargo --version          # Should be recent stable
cargo deny --version     # Should be installed
curl http://localhost:8000  # Should return DynamoDB Local info

# Verify clean state
git status              # Should be clean or on feature branch
cargo test --all-features  # Should pass all tests
```

## Implementation Steps

### Step 1: Update Cargo.toml Dependencies

**Objective:** Replace deprecated serde_yaml with serde_yml and update version constraints

**Actions:**

1.1. Open `Cargo.toml` in editor

1.2. Locate the dependencies section (around line 24-40)

1.3. Replace serde_yaml dependency:
```toml
# OLD (line 31):
serde_yaml = "0.9"

# NEW:
serde_yml = "0.0.12"
```

1.4. Review other dependencies for any updates needed:
- Keep all other dependencies at their current major version constraints
- All other deps are current within their semver ranges

1.5. Save `Cargo.toml`

**Verification:**
```bash
# Should show serde_yml instead of serde_yaml
grep -n "serde_y" Cargo.toml
```

**Expected Output:**
```
31:serde_yml = "0.0.12"
```

**Dependencies:** None

**Estimated Time:** 2 minutes

---

### Step 2: Update src/config.rs Import Statement

**Objective:** Replace serde_yaml import with serde_yml

**Actions:**

2.1. Open `src/config.rs` in editor

2.2. Locate the import statement (likely near the top of the file):
```rust
use serde_yaml;
```

2.3. Replace with:
```rust
use serde_yml;
```

2.4. Review any other references to `serde_yaml` in the file:
- Search for `serde_yaml::` and replace with `serde_yml::`
- Check function calls like `serde_yaml::from_reader()`
- Update to `serde_yml::from_reader()`

2.5. Save `src/config.rs`

**Verification:**
```bash
# Should show no occurrences of serde_yaml
grep -n "serde_yaml" src/config.rs

# Should show serde_yml usage
grep -n "serde_yml" src/config.rs
```

**Expected Output:**
```
# First command: no output (no serde_yaml found)
# Second command: shows line numbers with serde_yml
```

**Dependencies:** Step 1 must be completed first

**Estimated Time:** 3 minutes

---

### Step 3: Update src/error.rs Error Type

**Objective:** Update DynamoToolsError enum to use serde_yml::Error

**Actions:**

3.1. Open `src/error.rs` in editor

3.2. Locate the `DynamoToolsError` enum and find the `YamlParse` variant:
```rust
#[error("Failed to parse YAML from file '{file}': {source}")]
YamlParse {
    file: String,
    source: serde_yaml::Error,  // ← Update this
},
```

3.3. Replace with:
```rust
#[error("Failed to parse YAML from file '{file}': {source}")]
YamlParse {
    file: String,
    source: serde_yml::Error,  // ← Updated
},
```

3.4. Check if there are any From/Into implementations for serde_yaml::Error:
```bash
grep -n "serde_yaml::Error" src/error.rs
```

3.5. Update any found implementations to use `serde_yml::Error`

3.6. Save `src/error.rs`

**Verification:**
```bash
# Should show no occurrences of serde_yaml
grep -n "serde_yaml" src/error.rs

# Should show serde_yml::Error in YamlParse variant
grep -A 3 "YamlParse" src/error.rs
```

**Expected Output:**
```
YamlParse {
    file: String,
    source: serde_yml::Error,
},
```

**Dependencies:** Step 1 must be completed first

**Estimated Time:** 3 minutes

---

### Step 4: Update Cargo.lock

**Objective:** Lock new dependency versions and remove old serde_yaml

**Actions:**

4.1. Run cargo update to refresh dependency tree:
```bash
cargo update
```

4.2. Review the output to see what was updated:
- Should show serde_yaml being removed
- Should show serde_yml being added
- May show other patch version updates

4.3. Verify Cargo.lock changes:
```bash
# Check that serde_yaml is removed
grep -c "serde_yaml" Cargo.lock

# Check that serde_yml is added
grep -c "serde_yml" Cargo.lock
```

**Verification:**
```bash
# Should return 0 (no serde_yaml)
grep -c "^name = \"serde_yaml\"" Cargo.lock

# Should return 1 (serde_yml present)
grep -c "^name = \"serde_yml\"" Cargo.lock
```

**Expected Output:**
```
0  # serde_yaml
1  # serde_yml
```

**Dependencies:** Steps 1, 2, and 3 must be completed first

**Estimated Time:** 2 minutes

---

### Step 5: Verify Compilation

**Objective:** Ensure code compiles without errors or warnings

**Actions:**

5.1. Clean build artifacts:
```bash
cargo clean
```

5.2. Check compilation with all features:
```bash
cargo check --all-features
```

5.3. Review any errors or warnings:
- Should compile without errors
- Should have no new warnings
- Pay attention to any deprecation warnings

5.4. If errors occur:
- Review error messages carefully
- Common issues:
  - Missing trait implementations for serde_yml::Error
  - API differences between serde_yaml and serde_yml
- Fix errors and repeat Step 5.2

**Verification:**
```bash
cargo check --all-features 2>&1 | tee check-output.txt
echo "Exit code: $?"
```

**Expected Output:**
```
   Compiling dynamodb-tools v0.5.0
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
Exit code: 0
```

**Dependencies:** Steps 1-4 must be completed first

**Estimated Time:** 3 minutes

---

### Step 6: Run Clippy Lints

**Objective:** Ensure no new linting issues introduced

**Actions:**

6.1. Run clippy with all features and strict settings:
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

6.2. Review clippy output:
- Should pass with no warnings
- Project already allows `result_large_err` lint
- Look for any new warnings related to serde_yml

6.3. If warnings occur:
- Evaluate if they're related to the dependency change
- Fix legitimate issues
- Add `#[allow(clippy::...)]` only if necessary and justified

**Verification:**
```bash
cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tee clippy-output.txt
echo "Exit code: $?"
```

**Expected Output:**
```
    Finished dev [unoptimized + debuginfo] target(s)
Exit code: 0
```

**Dependencies:** Step 5 must be completed first

**Estimated Time:** 3 minutes

---

### Step 7: Run Full Test Suite

**Objective:** Verify all tests pass with new dependencies

**Actions:**

7.1. Ensure DynamoDB Local is running:
```bash
curl http://localhost:8000
```

7.2. Run all tests with all features:
```bash
cargo test --all-features -- --test-threads=1
```
Note: Using `--test-threads=1` prevents parallel test conflicts with DynamoDB Local

7.3. Review test results:
- All 6 integration tests should pass
- Pay special attention to YAML parsing tests:
  - `dev_config_should_create_and_describe_table`
  - `prod_config_should_return_empty_map_without_creating`
  - `multi_table_config_should_create_all_tables`
  - `dev_config_should_seed_data`
  - `simple_pk_table_should_allow_put`

7.4. If tests fail:
- Review failure messages carefully
- Check for YAML parsing errors (might indicate serde_yml incompatibility)
- Verify DynamoDB Local is accessible
- Check fixture files are intact

7.5. Optional: Run with nextest if available:
```bash
cargo nextest run --all-features
```

**Verification:**
```bash
cargo test --all-features -- --test-threads=1 2>&1 | tee test-output.txt
grep -E "(test result|FAILED)" test-output.txt
```

**Expected Output:**
```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Dependencies:** Steps 1-6 must be completed first; DynamoDB Local must be running

**Estimated Time:** 5 minutes

---

### Step 8: Run Cargo Deny Checks

**Objective:** Verify no license or security issues with new dependencies

**Actions:**

8.1. Run cargo-deny to check licenses:
```bash
cargo deny check licenses
```

8.2. Run cargo-deny to check for security advisories:
```bash
cargo deny check advisories
```

8.3. Run cargo-deny to check for banned dependencies:
```bash
cargo deny check bans
```

8.4. Review any warnings or errors:
- serde_yml should be MIT OR Apache-2.0 (same as serde_yaml)
- No new security advisories should appear
- No banned crates should be introduced

8.5. If issues occur:
- Review deny.toml configuration
- Check if serde_yml meets license requirements
- Investigate any security advisories

**Verification:**
```bash
cargo deny check 2>&1 | tee deny-output.txt
echo "Exit code: $?"
```

**Expected Output:**
```
advisories ok
licenses ok
bans ok
sources ok
Exit code: 0
```

**Dependencies:** Steps 1-4 must be completed first

**Estimated Time:** 2 minutes

---

### Step 9: Format Code

**Objective:** Ensure code formatting is consistent

**Actions:**

9.1. Run cargo fmt on all files:
```bash
cargo fmt --all
```

9.2. Verify no changes were made (code should already be formatted):
```bash
git diff src/
```

9.3. If changes appear:
- Review changes to ensure they're formatting-only
- Commit formatting changes separately if significant

**Verification:**
```bash
cargo fmt --all -- --check
echo "Exit code: $?"
```

**Expected Output:**
```
Exit code: 0
```

**Dependencies:** Steps 1-3 must be completed first

**Estimated Time:** 1 minute

---

### Step 10: Update Documentation

**Objective:** Document the dependency changes for users and maintainers

**Actions:**

10.1. Update CHANGELOG.md:
```bash
# Add entry under "Unreleased" or new version section
```

Example entry:
```markdown
## [Unreleased]

### Changed
- Migrated from deprecated `serde_yaml` (0.9.34+deprecated) to `serde_yml` (0.0.12)
- Updated all dependencies to latest patch versions within semver constraints
- No breaking changes to public API or YAML configuration format
```

10.2. Check if .claude/CLAUDE.md needs updates:
```bash
grep -n "serde_yaml" .claude/CLAUDE.md
```

10.3. If found, update references to serde_yml

10.4. Optional: Update README.md if it mentions dependencies:
```bash
grep -n "serde_yaml" README.md
```

10.5. Verify documentation accuracy:
- Ensure version numbers are correct
- Check that migration notes are clear
- Confirm no breaking changes mentioned

**Verification:**
```bash
# Review changes
git diff CHANGELOG.md .claude/CLAUDE.md README.md
```

**Expected Output:**
- CHANGELOG.md updated with clear entry
- Other docs updated if they referenced serde_yaml

**Dependencies:** Steps 1-9 should be completed first

**Estimated Time:** 5 minutes

---

### Step 11: Verify CI Configuration

**Objective:** Ensure GitHub Actions workflow will pass with changes

**Actions:**

11.1. Review `.github/workflows/build.yml`:
```bash
cat .github/workflows/build.yml
```

11.2. Verify the workflow steps:
- DynamoDB Local setup (rrainn/dynamodb-action@v2.0.1) ✓
- Rust toolchain installation ✓
- cargo fmt check ✓
- cargo check ✓
- cargo clippy ✓
- cargo nextest run ✓

11.3. No changes needed to workflow - it will work with updated dependencies

11.4. Optional: Consider updating GitHub Actions versions:
- `actions/checkout@v3` → `actions/checkout@v4`
- `Swatinem/rust-cache@v1` → `Swatinem/rust-cache@v2`
- Note: This is out of scope for dependency update, can be separate PR

**Verification:**
```bash
# Check workflow syntax
cat .github/workflows/build.yml | grep -E "(uses:|run:)" | head -20
```

**Expected Output:**
- Workflow file unchanged
- All steps present and correct

**Dependencies:** None (verification only)

**Estimated Time:** 2 minutes

---

### Step 12: Final Local Validation

**Objective:** Run complete validation suite before committing

**Actions:**

12.1. Run format check:
```bash
cargo fmt -- --check
```

12.2. Run clippy:
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

12.3. Run full test suite:
```bash
cargo test --all-features -- --test-threads=1
```

12.4. Run cargo deny:
```bash
cargo deny check
```

12.5. Build release version:
```bash
cargo build --release --all-features
```

12.6. Verify all steps pass without errors

**Verification:**
```bash
# Run all checks in sequence
cargo fmt -- --check && \
cargo clippy --all-features --all-targets -- -D warnings && \
cargo test --all-features -- --test-threads=1 && \
cargo deny check && \
cargo build --release --all-features

echo "All checks passed: $?"
```

**Expected Output:**
```
All checks passed: 0
```

**Dependencies:** All previous steps must be completed

**Estimated Time:** 5 minutes

---

## Step Dependencies Graph

```
Step 1: Update Cargo.toml
   ├─> Step 2: Update config.rs
   ├─> Step 3: Update error.rs
   └─> Step 4: Update Cargo.lock
          └─> Step 5: Verify Compilation
                 └─> Step 6: Run Clippy
                        └─> Step 7: Run Tests (requires DynamoDB Local)
                               └─> Step 8: Run Cargo Deny
                                      └─> Step 9: Format Code
                                             └─> Step 10: Update Docs
                                                    └─> Step 11: Verify CI Config
                                                           └─> Step 12: Final Validation
```

**Parallel Opportunities:**
- Steps 2 and 3 can be done in parallel (both depend only on Step 1)
- Steps 8 and 9 can be done in parallel (both depend on earlier steps)
- Step 11 is independent and can be done anytime after Step 1

## Verification Checkpoints

### Checkpoint 1: After Step 4 (Dependency Update Complete)
```bash
# Verify serde_yaml is completely removed
! grep -r "serde_yaml" src/ Cargo.toml Cargo.lock

# Verify serde_yml is present
grep -r "serde_yml" src/ Cargo.toml Cargo.lock | wc -l
# Should return > 3 (at least Cargo.toml, config.rs, error.rs)
```

### Checkpoint 2: After Step 6 (Code Quality)
```bash
# All quality checks pass
cargo fmt -- --check && \
cargo clippy --all-features --all-targets -- -D warnings
```

### Checkpoint 3: After Step 7 (Functionality)
```bash
# All tests pass
cargo test --all-features -- --test-threads=1
# Should show: test result: ok. 6 passed
```

### Checkpoint 4: After Step 12 (Ready to Commit)
```bash
# Complete validation suite
cargo fmt -- --check && \
cargo clippy --all-features --all-targets -- -D warnings && \
cargo test --all-features -- --test-threads=1 && \
cargo deny check && \
cargo build --release --all-features
```

## Rollback Procedure

If issues are encountered at any step:

1. **Before committing changes:**
   ```bash
   git restore src/ Cargo.toml Cargo.lock
   cargo clean
   cargo build --all-features
   cargo test --all-features
   ```

2. **After committing but before pushing:**
   ```bash
   git reset --hard HEAD~1
   cargo clean
   cargo build --all-features
   ```

3. **After pushing:**
   ```bash
   git revert <commit-hash>
   # Or create a new PR with the revert
   ```

## Success Criteria Summary

- [ ] All source files updated (config.rs, error.rs)
- [ ] Cargo.toml and Cargo.lock updated
- [ ] Code compiles without errors or warnings
- [ ] Clippy passes with no warnings
- [ ] All 6 integration tests pass
- [ ] cargo-deny passes all checks
- [ ] Code is formatted correctly
- [ ] Documentation updated (CHANGELOG.md minimum)
- [ ] No references to serde_yaml remain in codebase
- [ ] CI workflow verified (no changes needed)
- [ ] Ready for commit and PR submission

## Estimated Total Time

**Sequential execution:** ~30-35 minutes
**With parallel steps:** ~25-30 minutes

**Breakdown:**
- Code changes (Steps 1-4): 10 minutes
- Compilation and quality (Steps 5-6, 9): 7 minutes
- Testing (Step 7): 5 minutes
- Validation (Steps 8, 11-12): 9 minutes
- Documentation (Step 10): 5 minutes

**Note:** Times assume no issues encountered. Add 30-60 minutes buffer for troubleshooting if needed.
