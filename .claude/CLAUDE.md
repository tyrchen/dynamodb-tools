# Project Constitution: dynamodb-tools

## Project Overview

**dynamodb-tools** is a Rust library that simplifies AWS DynamoDB Local integration for development and testing. It provides YAML-based table schema definition, automatic unique table naming, seed data loading, and optional automatic cleanup.

- **Language**: Rust (Edition 2024)
- **Version**: 0.5.0
- **License**: MIT
- **Repository**: https://github.com/tyrchen/dynamodb-tools
- **Purpose**: Development and testing toolkit for DynamoDB Local
- **Type**: Library crate

### Key Features
- Declarative table schemas via YAML configuration
- Multi-table management from single config
- Unique table naming to prevent test pollution (base_name + xid)
- JSON-based seed data loading with batch writes
- Optional automatic table cleanup via Drop trait (test_utils feature)
- Full DynamoDB schema support: PK, SK, GSI, LSI, throughput

### Architecture
- **src/lib.rs** (12 LoC) - Library entry point with feature-gated exports
- **src/config.rs** (445 LoC) - YAML configuration parsing and AWS SDK type conversions
- **src/connector.rs** (256 LoC) - DynamoDB client wrapper with lifecycle management
- **src/error.rs** (54 LoC) - Custom error types via thiserror
- **tests/** - Integration tests requiring DynamoDB Local on localhost:8000
- **fixtures/** - Example YAML configs and JSON seed data

### Features
```toml
default = ["connector"]
connector = ["aws-config", "xid"]     # Main functionality
test_utils = ["tokio"]                # Auto-cleanup on drop
```

## Commands

### Prerequisites
```bash
# Start DynamoDB Local (required for tests)
java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib \
     -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar \
     -inMemory -sharedDb
```

### Build
```bash
cargo build                    # Standard build
cargo build --all-features     # Build with all features
cargo build --release          # Release build
make build                     # Via Makefile
```

### Test
```bash
cargo test --all-features              # Run all tests (needs DynamoDB Local)
cargo nextest run --all-features       # Run with nextest
cargo llvm-cov nextest --all-features  # Run with coverage
make test                              # Via Makefile

# Individual test files
cargo test --test connector_integration_test --all-features
```

**Important**: Integration tests require DynamoDB Local running on port 8000

### Lint & Format
```bash
cargo fmt                                # Format code
cargo fmt --check                        # Check formatting
cargo clippy --all-features              # Run clippy
cargo clippy --all-features -- -D warnings  # Clippy as errors
cargo check --all-features               # Check compilation
```

### Pre-commit Hooks
```bash
pre-commit install           # Install hooks
pre-commit run --all-files   # Run all hooks manually
```

Hooks: cargo fmt, cargo deny check, typos, cargo check, cargo clippy, cargo test

### Release
```bash
make release   # Generate changelog, tag, and push
cargo publish  # Publish to crates.io
```

### CI/CD
GitHub Actions workflow (`.github/workflows/build.yml`):
- Runs on push to master and PRs
- Sets up DynamoDB Local via rrainn/dynamodb-action
- Runs format check, clippy, and tests with nextest
- Environment: `AWS_DEFAULT_REGION=us-east-1`

## Project Structure

```
dynamodb-tools/
├── src/
│   ├── lib.rs           # Entry point with feature-gated exports
│   ├── config.rs        # TableConfig, TableInfo, YAML parsing, AWS conversions
│   ├── connector.rs     # DynamodbConnector, table lifecycle, seed data
│   └── error.rs         # DynamoToolsError enum with thiserror
├── tests/
│   └── connector_integration_test.rs  # Integration tests
├── fixtures/
│   ├── dev.yml          # Full development config example
│   ├── prod.yml         # Empty production config
│   ├── multi_table.yml  # Multi-table example
│   ├── info.yml         # Single table example
│   └── seed_users.json  # Sample seed data
├── .github/workflows/
│   └── build.yml        # CI/CD pipeline
├── Cargo.toml           # Package manifest
├── Makefile             # Build automation
├── cliff.toml           # Changelog generation
├── deny.toml            # Dependency auditing
└── .pre-commit-config.yaml  # Pre-commit hooks
```

### Module Organization
- **lib.rs**: Public API surface (`TableConfig`, `TableInfo`, `DynamodbConnector`, error types)
- **config.rs**: Configuration data structures and YAML/AWS SDK conversions
- **connector.rs**: AWS client wrapper with table creation, seeding, and cleanup
- **error.rs**: Comprehensive error handling with context

### Key Data Structures

```rust
// Configuration root
TableConfig {
    region: String,
    endpoint: String,
    delete_on_exit: bool,
    tables: Vec<TableInfo>
}

// Individual table schema
TableInfo {
    table_name: String,
    pk: TableAttr,
    sk: Option<TableAttr>,
    attrs: Vec<TableAttr>,
    gsis: Vec<TableGsi>,
    lsis: Vec<TableLsi>,
    throughput: Option<Throughput>,
    seed_data_file: Option<String>
}

// Connector with table mapping
DynamodbConnector {
    client: Client,
    table_mapping: HashMap<String, String>,  // base -> unique
    #[cfg(feature = "test_utils")]
    config: TableConfig
}
```

## Context-Aware Guidelines

When working on this project, follow these domain-specific guidelines:

### Rust Development
- **General**: Follow `~/.tda/constitutions/rust.md`
- Apply idiomatic Rust patterns (RAII, type safety, error propagation)
- Use thiserror for error types with context
- Leverage From/TryFrom traits for type conversions
- Use async/await for all AWS SDK operations

### AWS SDK Integration
- Use aws-sdk-dynamodb v1 API patterns
- Always configure endpoint, region, and credentials
- Handle AWS SDK errors via custom error enum
- Use aws-config for SDK initialization with BehaviorVersion::latest

### Testing Practices
- **Integration Tests**: Require DynamoDB Local on localhost:8000
- Use `#[cfg(all(test, feature = "test_utils"))]` for cleanup tests
- Use tokio test runtime: `#[tokio::test]`
- Validate table creation, schema, seeding, and cleanup
- Test multi-table scenarios
- Return `anyhow::Result<()>` from tests

### Configuration Patterns
- Define table schemas in YAML files in `fixtures/`
- Use consistent attribute naming (pk, sk for keys)
- Support optional sort keys, GSIs, LSIs
- Document seed data format requirements (JSON array of objects)
- Validate unique attribute deduplication across keys

### Error Handling
- Use `DynamoToolsError` enum for all library errors
- Provide context in error variants (file paths, field names)
- Wrap std::io::Error, serde errors, AWS SDK errors
- Use `Result<T>` type alias for consistency

### Feature Gates
- **connector** (default): Main functionality requiring aws-config and xid
- **test_utils**: Add automatic cleanup; requires tokio runtime
- Use `#[cfg(feature = "...")]` for feature-specific code
- Document feature requirements in public API

### Code Quality
- Run `cargo fmt` before committing
- Keep clippy warnings clean with `-- -D warnings`
- Allow `result_large_err` clippy lint (configured in Cargo.toml)
- Use pre-commit hooks for automated checks
- Maintain high test coverage

### Documentation
- Include usage examples in README
- Use doc comments with examples on public APIs
- Mark examples as `no_run` if they require external services
- Document feature requirements
- Explain YAML configuration schema
- Provide JSON seed data format examples

### Async Patterns
- Use tokio runtime for async operations
- Spawn background threads for cleanup in Drop impl (test_utils)
- Handle async errors with proper context
- Use aws_config::load_defaults() for SDK initialization

### Batch Operations
- Process DynamoDB batch writes in chunks of 25 (AWS limit)
- Convert JSON items to AttributeValue via serde_dynamo
- Handle batch write errors with context

### Naming Conventions
- Base table names from YAML config
- Append xid for unique names: `{base}_{unique_id}`
- Store mapping in HashMap for retrieval
- Use get_created_table_name() to retrieve unique names

### Version Management
- Use git-cliff for CHANGELOG generation
- Follow conventional commits
- Update CHANGELOG.md before releases
- Tag releases with version numbers

### Dependencies
- Audit with cargo-deny
- Keep AWS SDK updated to latest v1
- Minimize dependency footprint
- Use feature flags for optional dependencies

## Project-Specific Conventions

### YAML Configuration Schema

```yaml
region: us-east-1
endpoint: http://localhost:8000
delete_on_exit: true  # Requires test_utils feature

tables:
  - table_name: users
    pk:
      name: pk
      type: S  # S (String), N (Number), B (Binary)
    sk:
      name: sk
      type: S
    attrs:  # Additional attributes for GSI/LSI
      - name: gsi1pk
        type: S
    gsis:
      - name: gsi1
        pk:
          name: gsi1pk
          type: S
        sk:
          name: gsi1sk
          type: S
        attrs:  # Projected attributes
          - pk
        throughput:  # Optional
          rcu: 5
          wcu: 5
    lsis:
      - name: lsi1
        pk:
          name: pk  # Must match table PK
          type: S
        sk:
          name: lsi1sk
          type: S
    throughput:  # Optional; omit for pay-per-request
      rcu: 5
      wcu: 5
    seed_data_file: fixtures/seed_users.json  # Optional
```

### JSON Seed Data Format

```json
[
  {
    "pk": "user_1",
    "sk": "profile",
    "name": "Alice",
    "email": "alice@example.com"
  },
  {
    "pk": "user_2",
    "sk": "profile",
    "name": "Bob",
    "email": "bob@example.com"
  }
]
```

### Usage Pattern

```rust
use dynamodb_tools::DynamodbConnector;
use aws_sdk_dynamodb::types::AttributeValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load from YAML config
    let connector = DynamodbConnector::load("config.yml").await?;

    // Get unique table name (appended with xid)
    let table_name = connector
        .get_created_table_name("users")
        .expect("users table created");

    // Use AWS SDK client
    let result = connector.client()?
        .get_item()
        .table_name(table_name)
        .key("pk", AttributeValue::S("user_1".into()))
        .key("sk", AttributeValue::S("profile".into()))
        .send()
        .await?;

    println!("Item: {:?}", result.item);

    Ok(())
    // Tables auto-deleted if delete_on_exit=true and test_utils enabled
}
```

### Testing Pattern

```rust
#[cfg(all(test, feature = "test_utils"))]
#[tokio::test]
async fn test_table_creation() -> anyhow::Result<()> {
    let connector = DynamodbConnector::load("fixtures/dev.yml").await?;

    let table_name = connector
        .get_created_table_name("users")
        .expect("users table created");

    // Table name should have unique suffix
    assert!(table_name.starts_with("users_"));

    // Verify table exists
    let desc = connector.client()?
        .describe_table()
        .table_name(&table_name)
        .send()
        .await?;

    assert_eq!(desc.table().unwrap().table_status(), &TableStatus::Active);

    Ok(())
    // Cleanup happens automatically via Drop
}
```

### Error Handling Pattern

```rust
use dynamodb_tools::{DynamoToolsError, Result};

pub async fn create_tables(config_path: &str) -> Result<DynamodbConnector> {
    let config = TableConfig::load_from_file(config_path)
        .map_err(|e| DynamoToolsError::ConfigRead {
            file: config_path.to_string(),
            source: e,
        })?;

    DynamodbConnector::try_new(config).await
}
```

### Common Tasks

#### Add a New Error Variant
1. Add variant to `DynamoToolsError` enum in `src/error.rs`
2. Use thiserror attributes for error message and source
3. Update error handling in calling code

#### Add New Configuration Field
1. Add field to struct in `src/config.rs` (e.g., `TableInfo`)
2. Update YAML parsing (serde derives handle this automatically)
3. Update conversion to AWS SDK types if needed
4. Add example to fixtures/
5. Update README documentation

#### Add Integration Test
1. Create test function in `tests/connector_integration_test.rs`
2. Use `#[cfg(all(test, feature = "test_utils"))]` and `#[tokio::test]`
3. Create fixture YAML/JSON in `fixtures/` if needed
4. Ensure DynamoDB Local is documented as requirement
5. Return `anyhow::Result<()>` for flexible error handling

#### Update AWS SDK Version
1. Update version in `Cargo.toml` for `aws-sdk-dynamodb` and `aws-config`
2. Update serde_dynamo feature to match: `aws-sdk-dynamodb+{version}`
3. Check for breaking changes in AWS SDK API
4. Run full test suite
5. Update CHANGELOG.md

## Development Workflow

1. **Setup**: Start DynamoDB Local before development
2. **Changes**: Make changes following conventions above
3. **Format**: Run `cargo fmt` to format code
4. **Lint**: Run `cargo clippy --all-features -- -D warnings`
5. **Test**: Run `cargo test --all-features` (requires DynamoDB Local)
6. **Pre-commit**: Let pre-commit hooks validate
7. **CI**: GitHub Actions validates on push/PR

## Common Patterns to Follow

### Type Conversions
Always implement From/TryFrom for AWS SDK types:
```rust
impl From<AttrType> for ScalarAttributeType {
    fn from(attr_type: AttrType) -> Self {
        match attr_type {
            AttrType::S => ScalarAttributeType::S,
            AttrType::N => ScalarAttributeType::N,
            AttrType::B => ScalarAttributeType::B,
        }
    }
}
```

### Async AWS Operations
Always use async/await with proper error handling:
```rust
pub async fn create_table(&self, table_info: &TableInfo) -> Result<String> {
    let input = CreateTableInput::try_from(table_info.clone())?;

    self.client
        .create_table()
        .set_table_name(input.table_name)
        // ... more settings
        .send()
        .await
        .map_err(|e| DynamoToolsError::TableCreation {
            table: table_info.table_name.clone(),
            source: e.into()
        })?;

    Ok(unique_table_name)
}
```

### Feature-Gated Cleanup
Use Drop with background tokio threads:
```rust
#[cfg(feature = "test_utils")]
impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        if self.config.delete_on_exit {
            let client = self.client.clone();
            let tables = self.table_mapping.values().cloned().collect::<Vec<_>>();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    for table in tables {
                        let _ = client.delete_table().table_name(&table).send().await;
                    }
                });
            });
        }
    }
}
```

## Notes

- This library is designed for **development and testing only**, not production
- Always run DynamoDB Local before tests
- Unique table naming prevents test pollution in parallel execution
- test_utils feature should be used in dev-dependencies, not default features
- YAML configuration provides declarative, version-controlled table schemas
- Seed data enables consistent test scenarios
- Automatic cleanup reduces test boilerplate and ensures clean state
