# Verification Plan: Update Dependencies to Latest Versions

## Overview

This document outlines the comprehensive verification strategy for validating the dependency update from `serde_yaml` to `serde_yml` and ensuring all other dependencies are at their latest compatible versions. The verification plan ensures zero regression in functionality, performance, and code quality.

## Test Strategy

### Testing Philosophy

1. **Comprehensive Coverage**: All existing tests must pass without modification
2. **Integration Focus**: Emphasize integration tests over unit tests (primary test suite)
3. **Real Environment**: Tests run against actual DynamoDB Local instance
4. **Quality Gates**: Multiple automated quality checks before acceptance
5. **Zero Breaking Changes**: Public API remains unchanged

### Testing Layers

```
┌─────────────────────────────────────────┐
│   Layer 4: CI/CD Pipeline               │  ← GitHub Actions validation
├─────────────────────────────────────────┤
│   Layer 3: Quality Gates                │  ← Clippy, fmt, deny checks
├─────────────────────────────────────────┤
│   Layer 2: Integration Tests            │  ← Full YAML config tests
├─────────────────────────────────────────┤
│   Layer 1: Compilation & Dependencies   │  ← cargo check, cargo tree
└─────────────────────────────────────────┘
```

## Pre-Implementation Verification

### Baseline Establishment

Before making any changes, establish baseline metrics:

```bash
# 1. Record current test execution time
time cargo test --all-features -- --test-threads=1 > baseline-tests.txt 2>&1

# 2. Record current dependency count
cargo tree --depth 1 > baseline-deps.txt

# 3. Record current binary size
cargo build --release --all-features
ls -lh target/release/libdynamodb_tools.rlib > baseline-size.txt

# 4. Record current clippy warnings (if any)
cargo clippy --all-features --all-targets -- -W clippy::all > baseline-clippy.txt 2>&1
```

**Success Criteria:**
- All tests currently pass (6/6)
- Zero clippy warnings with `-D warnings`
- Clean compilation
- Baseline metrics recorded for comparison

## Test Cases

### Category 1: YAML Configuration Parsing

#### Test Case 1.1: Simple Configuration Loading
**Objective:** Verify basic YAML file parsing works with serde_yml

**Test Method:** Existing test `dev_config_should_create_and_describe_table()`

**Steps:**
1. Load `fixtures/dev.yml` using `TableConfig::load_from_file()`
2. Verify TableConfig struct is populated correctly
3. Create DynamodbConnector from config
4. Verify table is created with correct unique name

**Expected Result:**
- No parsing errors
- TableConfig contains expected values
- Table name starts with "users-"
- Table is created successfully in DynamoDB Local

**Failure Indicators:**
- `DynamoToolsError::YamlParse` error
- Incorrect field values in TableConfig
- Table creation fails

**Command:**
```bash
cargo test dev_config_should_create_and_describe_table --all-features -- --exact
```

---

#### Test Case 1.2: Empty Configuration
**Objective:** Verify empty YAML configs are handled correctly

**Test Method:** Existing test `prod_config_should_return_empty_map_without_creating()`

**Steps:**
1. Load `fixtures/prod.yml` (empty tables list)
2. Create DynamodbConnector from config
3. Verify no tables are created

**Expected Result:**
- No parsing errors
- Empty table mapping returned
- No DynamoDB tables created

**Failure Indicators:**
- Parsing error on empty config
- Unexpected tables created
- Null pointer or Option unwrap errors

**Command:**
```bash
cargo test prod_config_should_return_empty_map_without_creating --all-features -- --exact
```

---

#### Test Case 1.3: Multi-Table Configuration
**Objective:** Verify complex YAML with multiple tables parses correctly

**Test Method:** Existing test `multi_table_config_should_create_all_tables()`

**Steps:**
1. Load `fixtures/multi_table.yml` (2 tables defined)
2. Create DynamodbConnector from config
3. Verify both tables are created
4. Verify both tables can be described

**Expected Result:**
- YAML with multiple table definitions parses correctly
- Both tables created with unique names
- Table mappings stored correctly
- Both tables accessible via DynamoDB client

**Failure Indicators:**
- Parsing error on array of tables
- Only one table created
- Table name mapping incorrect

**Command:**
```bash
cargo test multi_table_config_should_create_all_tables --all-features -- --exact
```

---

#### Test Case 1.4: Configuration with Seed Data Reference
**Objective:** Verify YAML with seed_data_file field parses correctly

**Test Method:** Existing test `dev_config_should_seed_data()`

**Steps:**
1. Load `fixtures/dev.yml` (includes seed_data_file field)
2. Verify seed file path is captured in TableInfo
3. Verify seed data is loaded and written to table
4. Query table to verify seeded items exist

**Expected Result:**
- YAML parses with seed_data_file field
- Seed data JSON is loaded
- Items written to DynamoDB
- Seeded items queryable (user_1/profile with name=Alice)

**Failure Indicators:**
- seed_data_file field not parsed
- Seed data not loaded
- Items not written to table
- GetItem returns empty result

**Command:**
```bash
cargo test dev_config_should_seed_data --all-features -- --exact
```

---

### Category 2: Error Handling

#### Test Case 2.1: Invalid YAML Syntax
**Objective:** Verify clear error messages for malformed YAML

**Test Method:** Create temporary test with invalid YAML

**Steps:**
1. Create temp YAML file with syntax error (e.g., unmatched brackets)
2. Attempt to load with `TableConfig::load_from_file()`
3. Verify error type is `DynamoToolsError::YamlParse`
4. Verify error message is helpful

**Expected Result:**
- Error returned (not panic)
- Error type is `DynamoToolsError::YamlParse`
- Error message includes file path
- Error message includes YAML parsing details from serde_yml

**Failure Indicators:**
- Panic instead of error
- Generic error message
- Missing file path in error
- Confusing error message

**Manual Test:**
```bash
# Create invalid YAML
echo "tables:\n  - table_name: test\n    pk:\n      invalid yaml {" > /tmp/invalid.yml

# Test in Rust code
cargo test -- --ignored test_invalid_yaml_error
```

---

#### Test Case 2.2: Missing Required Fields
**Objective:** Verify error when YAML is missing required fields

**Test Method:** Create temporary test with incomplete YAML

**Steps:**
1. Create temp YAML with missing required field (e.g., no pk defined)
2. Attempt to load with `TableConfig::load_from_file()`
3. Verify appropriate error is returned

**Expected Result:**
- Error returned (not panic)
- Error message indicates missing field
- Error is actionable

**Failure Indicators:**
- Panic on missing field
- Silent default value used incorrectly
- Unclear error message

**Manual Test:**
```rust
#[test]
fn test_missing_required_field() {
    let yaml = r#"
region: us-east-1
endpoint: http://localhost:8000
tables:
  - table_name: incomplete
    # Missing pk field
    "#;

    let result = serde_yml::from_str::<TableConfig>(yaml);
    assert!(result.is_err());
}
```

---

#### Test Case 2.3: Type Mismatches
**Objective:** Verify error when YAML types don't match expected schema

**Test Method:** Create temporary test with wrong types

**Steps:**
1. Create temp YAML with type mismatch (e.g., string for bool field)
2. Attempt to load with `TableConfig::load_from_file()`
3. Verify appropriate error is returned

**Expected Result:**
- Error returned with type mismatch details
- Error message is clear

**Failure Indicators:**
- Silent coercion to wrong type
- Panic on type mismatch
- Confusing error message

---

### Category 3: Functional Integration Tests

#### Test Case 3.1: Table Creation and Basic Operations
**Objective:** Verify end-to-end table lifecycle works

**Test Method:** Existing test `simple_pk_table_should_allow_put()`

**Steps:**
1. Create TableConfig programmatically (not from file)
2. Create DynamodbConnector
3. Get unique table name
4. Perform PutItem operation
5. Verify operation succeeds

**Expected Result:**
- Table created with unique name
- PutItem succeeds
- No errors or panics

**Failure Indicators:**
- Table creation fails
- PutItem returns error
- Table name mapping incorrect

**Command:**
```bash
cargo test simple_pk_table_should_allow_put --all-features -- --exact
```

---

#### Test Case 3.2: Seed Data Loading and Verification
**Objective:** Verify complete seed data workflow

**Test Method:** Existing test `dev_config_should_seed_data()`

**Additional Validation:**
1. Verify all seed items are loaded (not just one)
2. Verify data types are preserved (S, N, B attributes)
3. Verify nested structures if any

**Expected Result:**
- All items from fixtures/seed_users.json are loaded
- Item attributes match JSON source
- GetItem retrieves correct data

**Failure Indicators:**
- Only some items loaded
- Data type conversion errors
- Attribute values incorrect

---

#### Test Case 3.3: Table Cleanup (test_utils feature)
**Objective:** Verify automatic table deletion on Drop

**Test Method:** Create focused test for cleanup

**Steps:**
1. Enable test_utils feature
2. Create config with `delete_on_exit: true`
3. Create connector and tables
4. Record created table names
5. Drop connector
6. Attempt to describe tables (should fail)

**Expected Result:**
- Tables exist before drop
- Tables deleted after drop
- No errors during cleanup

**Failure Indicators:**
- Tables persist after drop
- Cleanup errors logged
- DynamoDB Local errors

**Manual Test:**
```rust
#[cfg(feature = "test_utils")]
#[tokio::test]
async fn test_cleanup_on_drop() {
    let config = TableConfig::load_from_file("fixtures/dev.yml").unwrap();
    let table_name = {
        let connector = DynamodbConnector::try_new(config).await.unwrap();
        let name = connector.get_created_table_name("users").unwrap();
        name.clone()
    }; // connector dropped here

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Verify table no longer exists
    // Should fail with ResourceNotFoundException
}
```

---

### Category 4: Dependency Integrity

#### Test Case 4.1: Dependency Tree Validation
**Objective:** Verify no unexpected transitive dependencies

**Test Method:** Compare dependency trees before and after

**Steps:**
1. Generate dependency tree: `cargo tree > deps-after.txt`
2. Compare with baseline: `diff baseline-deps.txt deps-after.txt`
3. Verify changes are expected:
   - serde_yaml removed
   - serde_yml added
   - Other deps may have patch updates

**Expected Result:**
- serde_yaml and its unique deps removed
- serde_yml and its deps added
- No unexpected new dependencies
- No major version changes for other deps

**Failure Indicators:**
- Unexpected new dependencies
- Major version bumps
- Dependency conflicts

**Command:**
```bash
cargo tree --depth 1 | diff baseline-deps.txt -
```

---

#### Test Case 4.2: License Compliance
**Objective:** Verify new dependency meets license requirements

**Test Method:** Run cargo-deny license check

**Steps:**
1. Run `cargo deny check licenses`
2. Verify serde_yml has acceptable license (MIT OR Apache-2.0)
3. Verify no new license violations

**Expected Result:**
- All licenses approved
- serde_yml is MIT OR Apache-2.0
- No license warnings or errors

**Failure Indicators:**
- serde_yml has incompatible license
- New license violations
- cargo-deny fails

**Command:**
```bash
cargo deny check licenses
```

---

#### Test Case 4.3: Security Audit
**Objective:** Verify no security advisories for new dependencies

**Test Method:** Run cargo-deny advisory check

**Steps:**
1. Update advisory database: `cargo deny fetch`
2. Run `cargo deny check advisories`
3. Verify no new security issues

**Expected Result:**
- No security advisories for serde_yml
- No new vulnerabilities introduced
- All advisories pass

**Failure Indicators:**
- Security advisory for serde_yml
- New CVEs introduced
- cargo-deny fails

**Command:**
```bash
cargo deny check advisories
```

---

### Category 5: Code Quality

#### Test Case 5.1: Formatting Standards
**Objective:** Verify code follows Rust formatting conventions

**Test Method:** Run rustfmt check

**Steps:**
1. Run `cargo fmt -- --check`
2. Verify no formatting changes needed

**Expected Result:**
- All files properly formatted
- No changes required

**Failure Indicators:**
- Formatting violations found
- Inconsistent style

**Command:**
```bash
cargo fmt -- --check
```

---

#### Test Case 5.2: Clippy Linting
**Objective:** Verify no new linting violations

**Test Method:** Run clippy with strict settings

**Steps:**
1. Run `cargo clippy --all-features --all-targets -- -D warnings`
2. Verify no warnings or errors
3. Compare with baseline

**Expected Result:**
- Zero warnings with `-D warnings`
- No new clippy issues
- Existing allowed lints unchanged

**Failure Indicators:**
- New clippy warnings
- New clippy errors
- Regression in code quality

**Command:**
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

---

#### Test Case 5.3: Documentation Validity
**Objective:** Verify code documentation is valid

**Test Method:** Build documentation

**Steps:**
1. Run `cargo doc --all-features --no-deps`
2. Verify no broken links
3. Verify no doc warnings

**Expected Result:**
- Documentation builds successfully
- No broken intra-doc links
- No documentation warnings

**Failure Indicators:**
- Documentation build fails
- Broken links
- Missing documentation

**Command:**
```bash
cargo doc --all-features --no-deps
```

---

## Quality Gates

### Gate 1: Compilation (Blocking)

**Criteria:**
- `cargo check --all-features` exits with code 0
- No compilation errors
- No compilation warnings

**Command:**
```bash
cargo check --all-features
```

**Proceed to Gate 2 only if this passes.**

---

### Gate 2: Code Quality (Blocking)

**Criteria:**
- `cargo fmt -- --check` passes
- `cargo clippy --all-features --all-targets -- -D warnings` passes
- No new linting violations

**Commands:**
```bash
cargo fmt -- --check && \
cargo clippy --all-features --all-targets -- -D warnings
```

**Proceed to Gate 3 only if this passes.**

---

### Gate 3: Unit and Integration Tests (Blocking)

**Criteria:**
- All 6 existing tests pass
- No test failures or panics
- Test execution time within 10% of baseline

**Command:**
```bash
cargo test --all-features -- --test-threads=1
```

**Required Output:**
```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

**Proceed to Gate 4 only if this passes.**

---

### Gate 4: Dependency Audit (Blocking)

**Criteria:**
- `cargo deny check` passes all checks
- No license violations
- No security advisories
- No banned dependencies

**Command:**
```bash
cargo deny check
```

**Required Output:**
```
advisories ok
licenses ok
bans ok
sources ok
```

**Proceed to Gate 5 only if this passes.**

---

### Gate 5: Build Release (Blocking)

**Criteria:**
- Release build completes successfully
- Binary size within 10% of baseline
- No release-specific errors

**Command:**
```bash
cargo build --release --all-features
```

**Proceed to Gate 6 only if this passes.**

---

### Gate 6: CI/CD Validation (Non-Blocking but Required before Merge)

**Criteria:**
- GitHub Actions workflow passes
- All CI checks green
- No CI-specific failures

**Method:**
- Push to branch and observe CI results
- Review GitHub Actions logs

**Required:** All CI jobs must pass before merging to master.

---

## Regression Testing

### Performance Regression

**Metrics to Track:**

1. **Test Execution Time**
   ```bash
   # Before
   time cargo test --all-features -- --test-threads=1

   # After
   time cargo test --all-features -- --test-threads=1

   # Should be within ±10%
   ```

2. **Compilation Time**
   ```bash
   # Before
   cargo clean && time cargo build --all-features

   # After
   cargo clean && time cargo build --all-features

   # Should be within ±20%
   ```

3. **Binary Size**
   ```bash
   # Before and after
   cargo build --release --all-features
   ls -lh target/release/libdynamodb_tools.rlib

   # Should be within ±10%
   ```

**Acceptance Criteria:**
- Test time: ±10% of baseline
- Compile time: ±20% of baseline
- Binary size: ±10% of baseline

**If exceeded:** Investigate but does not block merge unless extreme (>50% regression).

---

### Behavioral Regression

**YAML Parsing Behavior:**

Test that identical YAML produces identical results:

```rust
#[test]
fn yaml_parsing_consistency() {
    let yaml_content = include_str!("../fixtures/dev.yml");

    let config = serde_yml::from_str::<TableConfig>(yaml_content).unwrap();

    // Verify expected values
    assert_eq!(config.region, "us-east-1");
    assert_eq!(config.endpoint, Some("http://localhost:8000".to_string()));
    assert_eq!(config.tables.len(), 1);
    assert_eq!(config.tables[0].table_name, "users");
    // ... etc
}
```

**Error Message Consistency:**

Verify error messages remain helpful:

```rust
#[test]
fn error_message_quality() {
    let result = TableConfig::load_from_file("nonexistent.yml");

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();

    // Should include file path
    assert!(err_msg.contains("nonexistent.yml"));

    // Should be clear about the issue
    assert!(err_msg.contains("Failed to") || err_msg.contains("Error"));
}
```

---

## Acceptance Criteria

### Functional Acceptance

- [ ] All 6 existing integration tests pass
- [ ] YAML parsing works for all fixture files
- [ ] Table creation, seeding, and cleanup work as expected
- [ ] Error handling remains clear and actionable
- [ ] No behavioral changes in public API

### Quality Acceptance

- [ ] Cargo fmt passes
- [ ] Cargo clippy passes with `-D warnings`
- [ ] Cargo deny passes all checks
- [ ] Documentation builds without warnings
- [ ] Code coverage maintained or improved

### Performance Acceptance

- [ ] Test execution time within ±10% of baseline
- [ ] Compilation time within ±20% of baseline
- [ ] Binary size within ±10% of baseline

### Dependency Acceptance

- [ ] serde_yaml completely removed
- [ ] serde_yml added and used correctly
- [ ] No unexpected transitive dependencies
- [ ] All licenses approved
- [ ] No security advisories

### Documentation Acceptance

- [ ] CHANGELOG.md updated with migration notes
- [ ] No outdated references to serde_yaml
- [ ] Migration is transparent to users
- [ ] README accurate

### CI/CD Acceptance

- [ ] GitHub Actions workflow passes
- [ ] All CI checks green
- [ ] No CI-specific failures

---

## Test Execution Plan

### Phase 1: Local Development (Pre-Commit)

Execute in this order:

```bash
# 1. Compile check
cargo check --all-features

# 2. Format check
cargo fmt -- --check

# 3. Clippy
cargo clippy --all-features --all-targets -- -D warnings

# 4. Tests (requires DynamoDB Local)
cargo test --all-features -- --test-threads=1

# 5. Dependency audit
cargo deny check

# 6. Release build
cargo build --release --all-features

# 7. Documentation
cargo doc --all-features --no-deps
```

**Total estimated time:** 5-10 minutes

---

### Phase 2: Pre-Push Validation

Additional validation before pushing:

```bash
# Run all checks together
cargo fmt -- --check && \
cargo clippy --all-features --all-targets -- -D warnings && \
cargo test --all-features -- --test-threads=1 && \
cargo deny check && \
cargo build --release --all-features && \
cargo doc --all-features --no-deps

echo "All pre-push checks passed!"
```

---

### Phase 3: CI/CD Pipeline

GitHub Actions runs automatically on push:

1. Checkout code
2. Setup DynamoDB Local
3. Install Rust toolchain
4. Install cargo-llvm-cov and nextest
5. Cache dependencies
6. Run format check
7. Run cargo check
8. Run clippy
9. Run tests with nextest
10. Generate coverage (optional)

**Monitor:** https://github.com/tyrchen/dynamodb-tools/actions

---

### Phase 4: Post-Merge Monitoring

After merging to master:

1. **Immediate (within 1 hour):**
   - Verify CI passes on master
   - Check for any user-reported issues

2. **Short-term (1-7 days):**
   - Monitor GitHub issues for bug reports
   - Watch crates.io download stats
   - Review any new CI failures

3. **Long-term (1-4 weeks):**
   - Monitor for security advisories
   - Watch for serde_yml updates
   - Consider publishing new crate version

---

## Troubleshooting Guide

### Issue: Tests fail with YAML parsing error

**Symptoms:**
- `DynamoToolsError::YamlParse` in test output
- Tests that previously passed now fail

**Diagnosis:**
```bash
# Check error message details
cargo test dev_config_should_create_and_describe_table -- --nocapture

# Verify YAML fixture files unchanged
git diff fixtures/
```

**Potential Causes:**
1. serde_yml has different YAML parsing rules than serde_yaml
2. Fixture file corrupted
3. Error type mismatch

**Resolution:**
1. Review serde_yml documentation for differences
2. Check fixture file syntax
3. Update error handling if needed

---

### Issue: Compilation fails with trait errors

**Symptoms:**
- Trait bound errors involving serde_yml::Error
- "does not implement trait" errors

**Diagnosis:**
```bash
# Check specific error
cargo check --all-features 2>&1 | grep -A 10 "error\["
```

**Potential Causes:**
1. serde_yml::Error doesn't implement expected traits
2. Missing trait imports

**Resolution:**
1. Add required trait implementations
2. Update error wrapping code
3. Consider newtype pattern if needed

---

### Issue: Clippy warnings about error handling

**Symptoms:**
- New clippy warnings in error.rs
- Warnings about Result usage

**Diagnosis:**
```bash
cargo clippy --all-features -- -W clippy::all
```

**Resolution:**
1. Review specific warnings
2. Update error handling patterns
3. Add `#[allow]` only if justified

---

### Issue: CI fails but local tests pass

**Symptoms:**
- GitHub Actions shows failures
- Local tests all pass

**Diagnosis:**
1. Check CI logs for specific error
2. Compare CI environment to local
3. Check AWS credentials in CI

**Potential Causes:**
1. DynamoDB Local version difference
2. Timing issues in parallel execution
3. Environment variable differences

**Resolution:**
1. Review `.github/workflows/build.yml`
2. Match CI environment locally
3. Add retries for flaky tests if needed

---

## Summary Checklist

Before committing:
- [ ] All code changes complete
- [ ] Cargo.toml updated
- [ ] Source files updated (config.rs, error.rs)
- [ ] Local compilation successful
- [ ] All tests pass locally
- [ ] Clippy passes
- [ ] Format check passes
- [ ] Cargo deny passes
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

Before pushing:
- [ ] All pre-push checks pass
- [ ] Git branch is clean
- [ ] Commit message is clear
- [ ] No debug code left in

Before merging:
- [ ] CI pipeline passes
- [ ] Code review complete (if applicable)
- [ ] All acceptance criteria met
- [ ] Documentation verified

Post-merge:
- [ ] Monitor CI on master branch
- [ ] Watch for issues
- [ ] Plan crate version update if needed

---

## Verification Timeline

**Total estimated verification time:** 15-20 minutes

**Breakdown:**
- Gate 1 (Compilation): 2 minutes
- Gate 2 (Code Quality): 3 minutes
- Gate 3 (Tests): 5 minutes
- Gate 4 (Dependency Audit): 2 minutes
- Gate 5 (Release Build): 3 minutes
- Gate 6 (CI/CD): 5-10 minutes (async)

**Critical Path:** Gates 1-5 must complete before pushing. Gate 6 validates after push.
