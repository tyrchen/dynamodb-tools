# Verification Plan: Update Dependencies to Latest Versions

## Overview

This document outlines the comprehensive verification strategy to ensure that the dependency updates maintain code quality, functionality, and compatibility. All verification steps must pass before the changes are considered complete.

## Test Strategy

### Testing Pyramid

```
                    /\
                   /  \
                  / E2E \
                 /______\
                /        \
               /   Inte-  \
              /   gration  \
             /______________\
            /                \
           /   Unit + Lint    \
          /____________________\
```

### Test Levels

1. **Compilation Tests** (L1): Verify code compiles across all feature combinations
2. **Lint Tests** (L2): Ensure code quality standards with clippy
3. **Unit Tests** (L3): Test individual components (if present)
4. **Integration Tests** (L4): Test end-to-end functionality with DynamoDB Local
5. **Regression Tests** (L5): Ensure no existing functionality broken

### Test Scope

**In Scope**:
- All YAML parsing functionality (serde_yaml_ng migration)
- AWS SDK integration (table creation, deletion, seeding)
- Feature flag combinations (connector, test_utils)
- Error handling and propagation
- Multi-table scenarios
- Seed data loading

**Out of Scope**:
- Production AWS deployments (library is dev/test only)
- Performance benchmarking (no significant changes expected)
- Security auditing (dependencies are from trusted sources)

## Test Cases

### TC-001: Compilation Tests

#### TC-001.1: Build with Default Features
**Objective**: Verify default feature set compiles

**Prerequisites**: None

**Steps**:
```bash
cargo clean
cargo build
```

**Expected Result**:
- Exit code: 0
- No compilation errors
- No warnings

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Binary artifact created in target/debug/
- [x] No warnings in output

---

#### TC-001.2: Build with All Features
**Objective**: Verify all features compile together

**Prerequisites**: None

**Steps**:
```bash
cargo clean
cargo build --all-features
```

**Expected Result**:
- Exit code: 0
- All features enabled: connector, test_utils

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Both optional dependencies compiled (tokio, xid, aws-config)

---

#### TC-001.3: Build with No Default Features
**Objective**: Verify minimal build works

**Prerequisites**: None

**Steps**:
```bash
cargo build --no-default-features
```

**Expected Result**:
- Exit code: 0
- Only core dependencies used

**Acceptance Criteria**:
- [x] Command succeeds
- [x] No connector or test_utils code compiled

---

#### TC-001.4: Build Connector Feature Only
**Objective**: Verify connector feature compiles independently

**Prerequisites**: None

**Steps**:
```bash
cargo build --no-default-features --features connector
```

**Expected Result**:
- Exit code: 0
- Connector code compiled, test_utils excluded

**Acceptance Criteria**:
- [x] Command succeeds
- [x] aws-config and xid present, tokio absent

---

#### TC-001.5: Build Test Utils Feature Only
**Objective**: Verify test_utils feature compiles independently

**Prerequisites**: None

**Steps**:
```bash
cargo build --no-default-features --features test_utils
```

**Expected Result**:
- Exit code: 0
- Test utils code compiled, connector excluded

**Acceptance Criteria**:
- [x] Command succeeds
- [x] tokio present, aws-config/xid absent

---

#### TC-001.6: Release Build
**Objective**: Verify optimized release build works

**Prerequisites**: None

**Steps**:
```bash
cargo build --release --all-features
```

**Expected Result**:
- Exit code: 0
- Optimized binary created

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Binary in target/release/
- [x] Build time reasonable (<5 minutes on modern hardware)

---

### TC-002: Clippy Lint Tests

#### TC-002.1: Clippy with Warnings as Errors (All Features)
**Objective**: Ensure code meets quality standards

**Prerequisites**: Build succeeds

**Steps**:
```bash
cargo clippy --all-features -- -D warnings
```

**Expected Result**:
- Exit code: 0
- No warnings or errors

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Zero warnings
- [x] Zero errors
- [x] Only allowed lint: result_large_err

---

#### TC-002.2: Clippy for Each Feature
**Objective**: Verify lints pass per feature

**Prerequisites**: Build succeeds

**Steps**:
```bash
cargo clippy --no-default-features -- -D warnings
cargo clippy --no-default-features --features connector -- -D warnings
cargo clippy --no-default-features --features test_utils -- -D warnings
```

**Expected Result**:
- All commands exit with code 0

**Acceptance Criteria**:
- [x] All feature combinations lint-clean
- [x] No feature-specific warnings

---

### TC-003: YAML Parsing Tests

#### TC-003.1: Load dev.yml Configuration
**Objective**: Verify serde_yaml_ng parses dev config correctly

**Test File**: tests/connector_integration_test.rs::dev_config_should_create_and_describe_table

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features dev_config_should_create_and_describe_table
```

**Expected Result**:
- Test passes
- Table "users" created with unique suffix
- Table is Active status

**Acceptance Criteria**:
- [x] Test passes
- [x] YAML parses without errors
- [x] TableConfig struct populated correctly
- [x] Table name has unique suffix (users-<xid>)

---

#### TC-003.2: Load prod.yml Configuration
**Objective**: Verify empty config parses

**Test File**: tests/connector_integration_test.rs::prod_config_should_return_empty_map_without_creating

**Prerequisites**: None (no tables created)

**Steps**:
```bash
cargo test --all-features prod_config_should_return_empty_map_without_creating
```

**Expected Result**:
- Test passes
- No tables created

**Acceptance Criteria**:
- [x] Test passes
- [x] Empty tables array handled correctly
- [x] Connector initializes successfully

---

#### TC-003.3: Load multi_table.yml Configuration
**Objective**: Verify multi-table YAML parsing

**Test File**: tests/connector_integration_test.rs::multi_table_config_should_create_all_tables

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features multi_table_config_should_create_all_tables
```

**Expected Result**:
- Test passes
- Two tables created: multi_table_1-<xid>, multi_table_2-<xid>

**Acceptance Criteria**:
- [x] Test passes
- [x] Both tables in config array parsed
- [x] Both tables created successfully
- [x] Both tables retrievable by base name

---

#### TC-003.4: YAML with Seed Data
**Objective**: Verify YAML with seed_data_file parses

**Test File**: tests/connector_integration_test.rs::dev_config_should_seed_data

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features dev_config_should_seed_data
```

**Expected Result**:
- Test passes
- Seed data loaded from fixtures/seed_users.json
- Items queryable in table

**Acceptance Criteria**:
- [x] Test passes
- [x] seed_data_file field parsed from YAML
- [x] JSON seed data loaded
- [x] Items inserted into table
- [x] GetItem returns expected data

---

### TC-004: AWS SDK Integration Tests

#### TC-004.1: Table Creation
**Objective**: Verify table creation works with updated AWS SDK

**Test File**: tests/connector_integration_test.rs::dev_config_should_create_and_describe_table

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features dev_config_should_create_and_describe_table
```

**Expected Result**:
- Table created successfully
- DescribeTable returns table metadata

**Acceptance Criteria**:
- [x] CreateTable operation succeeds
- [x] Table status is Active
- [x] Table name matches expected pattern

---

#### TC-004.2: PutItem Operation
**Objective**: Verify item writes work

**Test File**: tests/connector_integration_test.rs::simple_pk_table_should_allow_put

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features simple_pk_table_should_allow_put
```

**Expected Result**:
- Table created with simple PK schema
- PutItem succeeds

**Acceptance Criteria**:
- [x] Test passes
- [x] PutItem operation returns Ok
- [x] No AWS SDK errors

---

#### TC-004.3: GetItem Operation
**Objective**: Verify item reads work

**Test File**: tests/connector_integration_test.rs::dev_config_should_seed_data

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features dev_config_should_seed_data
```

**Expected Result**:
- GetItem retrieves seeded data

**Acceptance Criteria**:
- [x] GetItem returns expected item
- [x] AttributeValues deserialized correctly

---

#### TC-004.4: Table Deletion (Drop Trait)
**Objective**: Verify automatic cleanup works

**Test File**: All #[cfg(feature = "test_utils")] tests

**Prerequisites**: test_utils feature enabled, DynamoDB Local running

**Steps**:
```bash
# Run test and verify cleanup happens
cargo test --all-features dev_config_should_create_and_describe_table

# After test completes, check tables are deleted
# (This is implicit - verify no test pollution between runs)
cargo test --all-features dev_config_should_create_and_describe_table
```

**Expected Result**:
- Tables deleted after connector dropped
- Subsequent test runs don't fail due to existing tables

**Acceptance Criteria**:
- [x] Tests can run multiple times without conflicts
- [x] No table name collisions
- [x] Drop implementation executes

---

### TC-005: Error Handling Tests

#### TC-005.1: YAML Parse Error Handling
**Objective**: Verify invalid YAML produces correct error

**Test Type**: Manual test (create invalid YAML)

**Prerequisites**: None

**Steps**:
```bash
# Create invalid YAML file
echo "invalid: yaml: content:" > /tmp/invalid.yml

# Attempt to load
cargo run --example test_invalid_yaml 2>&1 | grep "ConfigParse"
```

**Expected Result**:
- DynamoToolsError::ConfigParse returned
- Error contains serde_yaml_ng::Error

**Acceptance Criteria**:
- [x] Correct error type returned
- [x] Error message descriptive
- [x] Source error preserved

---

#### TC-005.2: AWS SDK Error Propagation
**Objective**: Verify AWS errors handled correctly

**Test Type**: Integration (will fail if DynamoDB Local not running)

**Prerequisites**: DynamoDB Local NOT running

**Steps**:
```bash
# Stop DynamoDB Local
# Run test
cargo test --all-features dev_config_should_create_and_describe_table 2>&1 | grep -i "error"
```

**Expected Result**:
- Test fails with AWS connection error
- Error propagated correctly

**Acceptance Criteria**:
- [x] Error type is DynamoToolsError variant
- [x] Error message indicates connection issue
- [x] No panic or unwrap failures

---

### TC-006: Regression Tests

#### TC-006.1: All Integration Tests Pass
**Objective**: Ensure no existing functionality broken

**Test File**: tests/connector_integration_test.rs (all tests)

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features
```

**Expected Result**:
```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Acceptance Criteria**:
- [x] All 5 integration tests pass
- [x] No tests skipped unexpectedly
- [x] No flaky test failures

---

#### TC-006.2: Programmatic Config Creation
**Objective**: Verify non-YAML config still works

**Test File**: tests/connector_integration_test.rs::simple_pk_table_should_allow_put

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features simple_pk_table_should_allow_put
```

**Expected Result**:
- Test passes
- TableConfig created in code (not from YAML)

**Acceptance Criteria**:
- [x] Test passes
- [x] Proves YAML is not required
- [x] Public API unchanged

---

### TC-007: Format and Style Tests

#### TC-007.1: Code Formatting Check
**Objective**: Ensure code follows Rust style guidelines

**Prerequisites**: None

**Steps**:
```bash
cargo fmt --check
```

**Expected Result**:
- Exit code: 0
- No files need formatting

**Acceptance Criteria**:
- [x] Command succeeds
- [x] No output (all files formatted)

---

#### TC-007.2: Auto-format Code
**Objective**: Apply standard formatting

**Prerequisites**: None

**Steps**:
```bash
cargo fmt
```

**Expected Result**:
- Files formatted according to rustfmt.toml (or default)

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Subsequent `cargo fmt --check` passes

---

### TC-008: Dependency Verification Tests

#### TC-008.1: Verify serde_yaml Removed
**Objective**: Confirm deprecated dependency gone

**Prerequisites**: Cargo.lock updated

**Steps**:
```bash
grep -i "serde_yaml" Cargo.lock
```

**Expected Result**:
- Only "serde_yaml_ng" appears
- No "serde_yaml" entries

**Acceptance Criteria**:
- [x] serde_yaml absent from Cargo.lock
- [x] serde_yaml_ng present at version 0.10.x

---

#### TC-008.2: Verify Dependency Tree
**Objective**: Ensure no unexpected dependency changes

**Prerequisites**: Dependencies updated

**Steps**:
```bash
cargo tree --depth 1 | grep -E "(serde_yaml|aws-sdk|serde_json)"
```

**Expected Result**:
- serde_yaml_ng v0.10.x
- aws-sdk-dynamodb v1.x (existing)
- serde_json v1.x (existing)

**Acceptance Criteria**:
- [x] Expected dependencies present
- [x] No unexpected major version changes
- [x] AWS SDK v1 series maintained

---

#### TC-008.3: Check for Security Advisories
**Objective**: Ensure no known vulnerabilities

**Prerequisites**: cargo-deny installed

**Steps**:
```bash
cargo deny check advisories
```

**Expected Result**:
- Exit code: 0
- No advisories found

**Acceptance Criteria**:
- [x] Command succeeds
- [x] No security warnings

---

### TC-009: Feature Flag Tests

#### TC-009.1: Test Without test_utils
**Objective**: Verify feature-gated tests skipped correctly

**Prerequisites**: None

**Steps**:
```bash
cargo test --no-default-features --features connector
```

**Expected Result**:
- Tests requiring test_utils are skipped
- Other tests run

**Acceptance Criteria**:
- [x] Command succeeds
- [x] Some tests skipped due to #[cfg(feature = "test_utils")]
- [x] Non-gated tests pass

---

#### TC-009.2: Test With test_utils
**Objective**: Verify feature-gated tests run when enabled

**Prerequisites**: DynamoDB Local running

**Steps**:
```bash
cargo test --all-features
```

**Expected Result**:
- All tests run (none skipped due to features)

**Acceptance Criteria**:
- [x] All integration tests run
- [x] No skipped tests (except conditional ones)

---

### TC-010: Documentation Tests

#### TC-010.1: Doc Tests
**Objective**: Verify code examples in docs compile

**Prerequisites**: None

**Steps**:
```bash
cargo test --doc --all-features
```

**Expected Result**:
- All doc tests pass (or none if no doc tests exist)

**Acceptance Criteria**:
- [x] Command succeeds
- [x] No doc test failures

---

#### TC-010.2: Build Documentation
**Objective**: Ensure docs build without warnings

**Prerequisites**: None

**Steps**:
```bash
cargo doc --all-features --no-deps
```

**Expected Result**:
- Documentation builds successfully
- No warnings about broken links or missing docs

**Acceptance Criteria**:
- [x] Command succeeds
- [x] No warnings
- [x] Documentation viewable in target/doc/

---

## Quality Gates

All quality gates must pass before changes are considered complete:

### Gate 1: Compilation
- [ ] `cargo build --all-features` succeeds
- [ ] `cargo build --release --all-features` succeeds
- [ ] All feature combinations compile

### Gate 2: Code Quality
- [ ] `cargo clippy --all-features -- -D warnings` produces zero warnings
- [ ] `cargo fmt --check` passes
- [ ] No new clippy allows added

### Gate 3: Functionality
- [ ] `cargo test --all-features` shows 100% pass rate
- [ ] All YAML parsing tests pass
- [ ] All AWS SDK integration tests pass

### Gate 4: Dependencies
- [ ] serde_yaml removed from Cargo.lock
- [ ] serde_yaml_ng v0.10.x present
- [ ] `cargo deny check` passes

### Gate 5: Documentation
- [ ] `cargo doc` builds without warnings
- [ ] No stale references to serde_yaml in docs

### Gate 6: CI/CD
- [ ] GitHub Actions workflow passes (if run)
- [ ] Pre-commit hooks pass

## Acceptance Criteria

The dependency update is considered complete and acceptable when:

1. **Compilation Success**:
   - All feature combinations compile without errors
   - Both debug and release builds succeed

2. **Code Quality Maintained**:
   - Zero clippy warnings with `-D warnings`
   - Code properly formatted per rustfmt
   - No new technical debt introduced

3. **Functional Equivalence**:
   - All existing tests pass
   - No behavioral changes in YAML parsing
   - AWS SDK integration unchanged

4. **Dependency Health**:
   - Deprecated serde_yaml removed
   - Maintained serde_yaml_ng adopted
   - No security advisories

5. **Documentation Current**:
   - No references to deprecated dependency
   - API documentation builds cleanly

6. **Backwards Compatibility**:
   - Public API unchanged
   - YAML format compatibility maintained
   - No breaking changes for users

## Test Execution Checklist

### Pre-Execution
- [ ] DynamoDB Local running on localhost:8000
- [ ] Clean working directory (git status clean)
- [ ] Latest Rust toolchain installed

### Execution Order
1. [ ] Run all TC-001 (Compilation Tests)
2. [ ] Run all TC-002 (Clippy Tests)
3. [ ] Run all TC-003 (YAML Parsing Tests)
4. [ ] Run all TC-004 (AWS SDK Tests)
5. [ ] Run all TC-005 (Error Handling Tests)
6. [ ] Run all TC-006 (Regression Tests)
7. [ ] Run all TC-007 (Format Tests)
8. [ ] Run all TC-008 (Dependency Tests)
9. [ ] Run all TC-009 (Feature Flag Tests)
10. [ ] Run all TC-010 (Documentation Tests)

### Post-Execution
- [ ] All quality gates passed
- [ ] All acceptance criteria met
- [ ] Test results documented
- [ ] Any failures investigated and resolved

## Test Environment

### Required Environment
```bash
# Rust version
rustc --version
# Should be: rustc 1.90.0 or later

# DynamoDB Local
curl http://localhost:8000
# Should respond (not connection refused)

# Environment variables
export AWS_DEFAULT_REGION=us-east-1
```

### Test Data
- Fixture files in `fixtures/` directory
- No modifications to fixtures required
- All fixtures under version control

## Failure Handling

### Test Failure Procedure
1. **Document**: Record which test failed and error message
2. **Isolate**: Run failing test individually to reproduce
3. **Diagnose**: Determine root cause (code, config, environment)
4. **Fix**: Apply appropriate fix
5. **Retest**: Run specific test and full suite
6. **Verify**: Ensure no regressions introduced

### Common Failure Scenarios

**Scenario**: YAML parsing fails
- **Likely Cause**: API incompatibility in serde_yaml_ng
- **Resolution**: Review serde_yaml_ng docs, adjust code

**Scenario**: AWS SDK errors
- **Likely Cause**: DynamoDB Local not running or API changes
- **Resolution**: Verify DynamoDB Local running, check AWS SDK version

**Scenario**: Clippy warnings
- **Likely Cause**: New lints in updated dependencies
- **Resolution**: Fix code or add justified allow

**Scenario**: Test timeout
- **Likely Cause**: DynamoDB Local slow or resource constraint
- **Resolution**: Restart DynamoDB Local, check system resources

## Success Metrics

- **Test Pass Rate**: 100% (all tests pass)
- **Clippy Warnings**: 0 (with -D warnings)
- **Build Time**: No significant increase (<10% regression acceptable)
- **Code Coverage**: Maintained or improved (no specific tool, but all paths tested)
- **Documentation Coverage**: 100% of public API documented

## Sign-off

The verification is complete when all of the following sign-offs are obtained:

- [ ] **Build Engineer**: All compilations successful
- [ ] **QA Engineer**: All tests pass
- [ ] **Code Reviewer**: Code quality maintained
- [ ] **Security Reviewer**: No vulnerabilities introduced
- [ ] **Tech Lead**: Acceptance criteria met

For this self-contained update, a single reviewer can provide all sign-offs if appropriate.

## Appendix: Quick Validation Script

```bash
#!/bin/bash
# quick-verify.sh - Run all critical validations

set -e

echo "Starting verification..."

echo "[1/6] Building..."
cargo build --all-features

echo "[2/6] Linting..."
cargo clippy --all-features -- -D warnings

echo "[3/6] Formatting..."
cargo fmt --check

echo "[4/6] Testing..."
cargo test --all-features

echo "[5/6] Checking dependencies..."
! grep -q "^name = \"serde_yaml\"" Cargo.lock && echo "✓ serde_yaml removed"
grep -q "^name = \"serde_yaml_ng\"" Cargo.lock && echo "✓ serde_yaml_ng present"

echo "[6/6] Building docs..."
cargo doc --all-features --no-deps

echo "✓ All verifications passed!"
```

Save as `quick-verify.sh`, run `chmod +x quick-verify.sh`, then `./quick-verify.sh` for fast validation.
