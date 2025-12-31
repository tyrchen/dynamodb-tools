# Design Document: Update Dependencies to Latest Versions

## Overview

This design document outlines the plan to update all dependencies in the `dynamodb-tools` Rust library to their latest compatible versions. The primary focus is on:

1. Migrating from deprecated `serde_yaml` (0.9.34+deprecated) to `serde_yml` (0.0.12)
2. Ensuring all other dependencies are at their latest stable versions
3. Maintaining backward compatibility with the public API
4. Ensuring all tests pass with the updated dependencies

## Current State Analysis

### Current Dependencies (from Cargo.toml)

**Production Dependencies:**
- `anyhow = "1"` → Currently 1.0.100 (latest: 1.0.x)
- `aws-config = "1"` → Currently 1.8.12 (latest: 1.x)
- `aws-sdk-dynamodb = "1"` → Currently 1.101.0 (latest: 1.x)
- `serde = "1"` → Currently 1.0.228 (latest: 1.0.x)
- `serde_yaml = "0.9"` → **DEPRECATED** at 0.9.34+deprecated
- `thiserror = "2"` → Currently 2.0.17 (latest: 2.0.x)
- `tokio = "1"` (optional) → Currently 1.48.0 (latest: 1.x)
- `xid = "1"` (optional) → Currently 1.1.1 (latest: 1.x)
- `tracing = "0.1"` → Currently 0.1.44 (latest: 0.1.x)
- `serde_dynamo = "4"` → Currently 4.3.0 (latest: 4.x)
- `serde_json = "1"` → Currently 1.0.148 (latest: 1.0.x)

**Dev Dependencies:**
- `serde_json = "1.0.140"` → Pin removed in favor of workspace version
- `tokio = "1"` → Currently 1.48.0 (latest: 1.x)

### Critical Issue: serde_yaml Deprecation

The `serde_yaml` crate is officially deprecated with version `0.9.34+deprecated`. The maintainer (dtolnay) has marked it as deprecated and recommends migrating to alternative YAML libraries.

**Recommended Migration Path:**
- **Target:** `serde_yml` version 0.0.12
- **Reason:** Active maintained fork/replacement with compatible API
- **Repository:** https://github.com/sebastienrousseau/serde_yml
- **License:** MIT OR Apache-2.0 (same as serde_yaml)

### Dependency Status Summary

Based on `cargo tree` output:
- All dependencies are at reasonable versions within their major version constraints
- `cargo update --dry-run` shows 0 packages would be updated within Rust 1.90.0 compatibility
- All dependencies use semantic versioning and are locked to major versions
- No security advisories detected (would show in cargo-deny)

### Code Usage Analysis

**serde_yaml Usage Locations:**
1. `src/config.rs` - Primary usage for YAML configuration parsing
   - `TableConfig::load_from_file()` - Deserializes YAML to TableConfig struct
   - Uses `serde_yaml::from_reader()` for file-based deserialization

**Impact Assessment:**
- Public API surface does NOT expose serde_yaml types
- Usage is internal to the `config.rs` module
- Migration should be transparent to library users
- No breaking changes to public API expected

## Technical Approach

### 1. Dependency Migration Strategy

#### Phase 1: Replace serde_yaml with serde_yml

**Code Changes Required:**
```rust
// Before (src/config.rs)
use serde_yaml;

// After (src/config.rs)
use serde_yml;
```

**API Compatibility:**
- `serde_yml::from_reader()` - Direct replacement for `serde_yaml::from_reader()`
- Error types may differ - needs verification
- Serialization behavior should be identical for standard YAML

**Error Handling:**
Current code wraps serde_yaml errors in `DynamoToolsError::YamlParse`:
```rust
pub enum DynamoToolsError {
    #[error("Failed to parse YAML from file '{file}': {source}")]
    YamlParse {
        file: String,
        source: serde_yaml::Error,  // ← Needs update
    },
    // ...
}
```

**Required Changes:**
- Update error type from `serde_yaml::Error` to `serde_yml::Error`
- Verify error message formatting remains user-friendly
- Ensure error context is preserved

#### Phase 2: Update Cargo.toml

```toml
[dependencies]
# Remove
# serde_yaml = "0.9"

# Add
serde_yml = "0.0.12"
```

#### Phase 3: Update Cargo.lock

- Run `cargo update` to update all dependencies within version constraints
- This will update patch versions for all dependencies
- Lock file will be updated automatically

### 2. Backward Compatibility

**Public API Guarantees:**
- No changes to public structs (`TableConfig`, `TableInfo`, `DynamodbConnector`)
- No changes to public methods or function signatures
- No changes to feature flags
- YAML configuration format remains unchanged

**User Impact:**
- Zero breaking changes for library users
- Existing YAML configuration files work without modification
- Existing code using this library requires no changes
- Re-compilation required (dependency change)

### 3. Testing Strategy

**Existing Test Coverage:**
- Integration tests in `tests/connector_integration_test.rs`
- Tests use YAML fixtures in `fixtures/` directory
- Tests require DynamoDB Local on localhost:8000

**Critical Tests:**
1. `dev_config_should_create_and_describe_table()` - Tests YAML parsing of dev.yml
2. `prod_config_should_return_empty_map_without_creating()` - Tests prod.yml
3. `multi_table_config_should_create_all_tables()` - Tests multi_table.yml
4. `dev_config_should_seed_data()` - Tests YAML with seed data reference

**Validation Points:**
- All existing tests must pass without modification
- YAML parsing behavior must be identical
- Error messages should remain clear and actionable
- No performance regressions

### 4. CI/CD Considerations

**GitHub Actions Workflow (`.github/workflows/build.yml`):**
- Uses `actions/checkout@v3` - Consider updating to v4
- Uses `rrainn/dynamodb-action@v2.0.1` - Current version OK
- Uses `Swatinem/rust-cache@v1` - Consider updating to v2
- Rust toolchain: stable with llvm-tools-preview
- Test environment: AWS credentials set for DynamoDB Local

**CI Updates Needed:**
- Potentially update GitHub Actions versions
- No changes to DynamoDB Local setup required
- No changes to test environment variables required

## Components Affected

### Source Files

1. **src/config.rs** (445 LoC)
   - **Changes:** Replace `serde_yaml` import with `serde_yml`
   - **Impact:** HIGH - Core configuration parsing logic
   - **Risk:** LOW - API is compatible

2. **src/error.rs** (54 LoC)
   - **Changes:** Update `YamlParse` error variant to use `serde_yml::Error`
   - **Impact:** MEDIUM - Error type definition
   - **Risk:** LOW - Error wrapping, no public API change

3. **Cargo.toml** (45 LoC)
   - **Changes:** Replace `serde_yaml = "0.9"` with `serde_yml = "0.0.12"`
   - **Impact:** HIGH - Dependency declaration
   - **Risk:** LOW - Standard dependency update

4. **Cargo.lock** (auto-generated)
   - **Changes:** Automatic update via `cargo update`
   - **Impact:** HIGH - All transitive dependencies
   - **Risk:** LOW - Locked versions ensure reproducibility

### Test Files

1. **tests/connector_integration_test.rs** (198 LoC)
   - **Changes:** None required (uses public API only)
   - **Impact:** NONE - Validation only
   - **Risk:** NONE

### Configuration Files

1. **fixtures/*.yml** (YAML configuration files)
   - **Changes:** None required
   - **Impact:** NONE - YAML format unchanged
   - **Risk:** NONE

### Documentation

1. **README.md**
   - **Changes:** Consider noting serde_yaml → serde_yml migration
   - **Impact:** LOW - Information only
   - **Risk:** NONE

2. **CHANGELOG.md**
   - **Changes:** Add entry for dependency updates
   - **Impact:** MEDIUM - User communication
   - **Risk:** NONE

3. **.claude/CLAUDE.md**
   - **Changes:** Update dependency list if it references serde_yaml
   - **Impact:** LOW - Development reference
   - **Risk:** NONE

## Risk Assessment

### High Risk Areas

**None identified.** This is a low-risk change due to:
- Internal-only dependency usage
- Compatible API replacement
- Comprehensive test coverage
- No public API changes

### Medium Risk Areas

1. **Error Message Changes**
   - **Risk:** Error messages from serde_yml may differ from serde_yaml
   - **Mitigation:** Review error output in tests; document any changes
   - **Impact:** Users may see slightly different error messages for YAML parsing failures

2. **YAML Parsing Edge Cases**
   - **Risk:** serde_yml may handle edge cases differently than serde_yaml
   - **Mitigation:** Run full test suite; test with various YAML configurations
   - **Impact:** Unlikely to affect standard use cases

### Low Risk Areas

1. **Transitive Dependency Updates**
   - **Risk:** `cargo update` may update transitive dependencies with behavior changes
   - **Mitigation:** Cargo.lock ensures reproducible builds; CI validates changes
   - **Impact:** Minimal - semver guarantees apply

2. **CI/CD Workflow**
   - **Risk:** Updated dependencies may expose CI configuration issues
   - **Mitigation:** Monitor CI after PR submission; easy to revert
   - **Impact:** CI-only, no production impact

### Mitigation Strategies

1. **Incremental Testing**
   - Update dependencies locally first
   - Run full test suite before committing
   - Verify DynamoDB Local integration

2. **Rollback Plan**
   - Git provides easy rollback via branch reset
   - No database migrations or state changes involved
   - Can revert to previous Cargo.toml/Cargo.lock

3. **Documentation**
   - Document the serde_yaml → serde_yml migration in CHANGELOG
   - Update any references in documentation
   - Note semantic versioning for future maintainers

4. **CI Validation**
   - Let GitHub Actions run full build and test suite
   - Review clippy warnings for any new issues
   - Ensure cargo-deny passes (license and security checks)

## Alternative Approaches Considered

### Option 1: Keep serde_yaml (NOT RECOMMENDED)
- **Pros:** No code changes required
- **Cons:** Using deprecated crate; no future updates; potential security issues
- **Verdict:** REJECTED - Not sustainable

### Option 2: Migrate to serde_yaml_ng
- **Pros:** Another maintained fork of serde_yaml
- **Cons:** Less actively maintained than serde_yml
- **Verdict:** REJECTED - serde_yml is more active

### Option 3: Switch to a different YAML library (e.g., yaml-rust2)
- **Pros:** Different implementation, potentially better performance
- **Cons:** Different API, more extensive code changes required
- **Verdict:** REJECTED - Unnecessary complexity

### Option 4: Selected Approach - Migrate to serde_yml
- **Pros:** Drop-in replacement, actively maintained, compatible API, same license
- **Cons:** Relatively new crate (0.0.12), smaller community
- **Verdict:** ACCEPTED - Best balance of compatibility and maintenance

## Success Criteria

1. All tests pass with updated dependencies
2. No clippy warnings introduced by dependency updates
3. cargo-deny passes (no license or security issues)
4. CI/CD pipeline succeeds
5. No breaking changes to public API
6. YAML parsing behavior remains consistent
7. Error messages remain clear and actionable
8. Documentation updated to reflect changes

## Timeline and Effort Estimate

**Estimated Effort:** 2-4 hours

**Breakdown:**
- Dependency update and code changes: 30 minutes
- Local testing and validation: 1 hour
- Documentation updates: 30 minutes
- CI validation and iteration: 1-2 hours
- Buffer for unexpected issues: 30 minutes

**No timeline dependencies** - This is a standalone maintenance task that can be completed in a single session.

## Post-Migration Monitoring

1. **Short-term (1-2 weeks):**
   - Monitor GitHub issues for user-reported problems
   - Watch for CI failures in main branch
   - Check crates.io download stats remain stable

2. **Long-term (1-3 months):**
   - Monitor serde_yml for updates
   - Watch for any security advisories
   - Consider additional dependency updates if needed

## Conclusion

This is a low-risk, high-value maintenance task that addresses a deprecated dependency and ensures the library remains maintainable. The migration from serde_yaml to serde_yml is straightforward with minimal code changes and no impact on library users. The approach prioritizes backward compatibility and thorough testing to ensure a smooth transition.
