# DynamoDB Tools - Repository Analysis

## Project Overview

**dynamodb-tools** is a Rust library that simplifies working with AWS DynamoDB Local for development and testing environments. Originally named `dynamodb-tester`, it was renamed to reflect its broader utility beyond just testing scenarios.

### Key Capabilities

- **Declarative Table Schema Definition**: Define DynamoDB table schemas in YAML configuration files
- **Multi-Table Management**: Create and manage multiple tables from a single configuration
- **Unique Table Naming**: Automatically generates unique table names (base_name + xid) to prevent test pollution
- **Seed Data Support**: Load initial test data from JSON files
- **Automatic Cleanup**: Optional automatic table deletion when connector goes out of scope (via `test_utils` feature)
- **Full Schema Support**: Primary keys, sort keys, GSIs (Global Secondary Indexes), LSIs (Local Secondary Indexes)

### Project Metadata

- **Version**: 0.5.0
- **License**: MIT
- **Repository**: https://github.com/tyrchen/dynamodb-tools
- **Documentation**: https://docs.rs/dynamodb-tools
- **Category**: Development Tools
- **Keywords**: aws-sdk, dynamodb, testing

## Tech Stack

### Language & Edition
- **Rust Edition**: 2024 (Note: This appears to be a typo in Cargo.toml - should likely be 2021)
- **Minimum Rust Version**: Not explicitly specified

### Core Dependencies

#### AWS SDK Integration
- `aws-sdk-dynamodb` (v1) - AWS DynamoDB SDK for Rust
- `aws-config` (v1, optional) - AWS configuration and credential management
  - Feature: `behavior-version-latest`

#### Serialization & Data Handling
- `serde` (v1) - Serialization framework with derive macros
- `serde_yaml` (v0.9) - YAML parsing for configuration files
- `serde_json` (v1) - JSON parsing for seed data
- `serde_dynamo` (v4) - DynamoDB AttributeValue serialization/deserialization
  - Feature: `aws-sdk-dynamodb+1`

#### Error Handling & Utilities
- `thiserror` (v2) - Derive-based error handling
- `anyhow` (v1) - Flexible error handling
- `tracing` (v0.1) - Observability and structured logging
- `xid` (v1, optional) - Unique ID generation for table naming

#### Async Runtime
- `tokio` (v1, optional) - Async runtime for background cleanup
  - Features: `macros`, `rt-multi-thread`

### Development Dependencies
- `serde_json` (v1.0.140) - JSON handling in tests
- `tokio` (v1) - Async runtime for integration tests

### Build & Development Tools
- **CI/CD**: GitHub Actions (`.github/workflows/build.yml`)
- **Pre-commit Hooks**: Formatting, linting, testing, spell-checking
- **Code Coverage**: cargo-llvm-cov
- **Test Runner**: cargo-nextest
- **Changelog**: git-cliff (cliff.toml)
- **Dependency Auditing**: cargo-deny (deny.toml)

## Directory Structure

```
dynamodb-tools/
├── src/                              # Source code (964 LoC total)
│   ├── lib.rs                       # Library entry point (12 LoC)
│   ├── config.rs                    # Configuration structures (445 LoC)
│   ├── connector.rs                 # DynamoDB connector implementation (256 LoC)
│   └── error.rs                     # Error types and handling (54 LoC)
│
├── tests/                            # Integration tests
│   └── connector_integration_test.rs # Integration tests (197 LoC)
│
├── fixtures/                         # Test data and configurations
│   ├── dev.yml                      # Development config with full schema
│   ├── prod.yml                     # Production config (empty tables)
│   ├── multi_table.yml              # Multi-table example
│   ├── info.yml                     # Single table info example
│   └── seed_users.json              # Sample seed data
│
├── .github/workflows/               # CI/CD pipelines
│   └── build.yml                    # GitHub Actions workflow
│
├── .vscode/                         # VS Code settings
├── Cargo.toml                       # Package manifest
├── Cargo.lock                       # Dependency lock file
├── Makefile                         # Build automation
├── README.md                        # User documentation
├── CHANGELOG.md                     # Version history
├── LICENSE.md                       # MIT License
├── cliff.toml                       # Changelog generator config
├── deny.toml                        # Dependency audit rules
└── .pre-commit-config.yaml          # Pre-commit hook configuration
```

## Architecture

### Module Overview

#### `src/lib.rs` - Library Entry Point
- Includes README as documentation via `include_str!`
- Feature-gated module exports
- Public API: `AttrType`, `TableAttr`, `TableConfig`, `TableInfo`, `DynamodbConnector`, error types

#### `src/error.rs` - Error Handling (54 lines)
Custom error type `DynamoToolsError` using `thiserror`:
- `ConfigRead` - File I/O errors reading configuration
- `ConfigParse` - YAML parsing errors
- `AwsSdkConfig` - AWS SDK configuration errors
- `MissingField` - Missing required fields in config/responses
- `DynamoDbSdk` - Generic AWS SDK errors
- `TableCreation` - Table creation failures
- `TableDeletion` - Table deletion failures
- `TableDescribe` - Table describe failures
- `SeedFileRead` - Seed data file I/O errors
- `SeedJsonParse` - JSON parsing errors in seed files
- `SeedDynamoConversion` - serde_dynamo conversion errors
- `SeedBatchWrite` - Batch write operation failures
- `Internal` - Internal/unexpected errors

#### `src/config.rs` - Configuration Layer (445 lines)

**Data Structures:**
```rust
TableConfig {
    region: String,
    endpoint: String,
    delete_on_exit: bool,
    tables: Vec<TableInfo>
}

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

TableAttr { name: String, type: AttrType }
AttrType { S | N | B }  // String, Number, Binary

TableGsi {
    name: String,
    pk: TableAttr,
    sk: Option<TableAttr>,
    attrs: Vec<String>,
    throughput: Option<Throughput>
}

TableLsi {
    name: String,
    pk: TableAttr,
    sk: TableAttr,  // Required for LSI
    attrs: Vec<String>
}

Throughput { rcu: i64, wcu: i64 }
```

**Key Features:**
- Unique attribute deduplication across table keys, GSI keys, and LSI keys
- YAML-based configuration loading via `load_from_file()` and `load()`
- Conversion to AWS SDK types via `From` and `TryFrom` traits
- Programmatic configuration creation via `new()`

#### `src/connector.rs` - DynamoDB Connector (256 lines)

**Core Type:**
```rust
DynamodbConnector {
    client: Client,
    table_mapping: HashMap<String, String>,  // base_name -> unique_name
    #[cfg(feature = "test_utils")]
    config: TableConfig
}
```

**Key Methods:**
- `load(file_path)` - Load connector from configuration file
- `try_new(config)` - Create connector from TableConfig
- `client()` - Access underlying AWS DynamoDB client
- `get_created_table_name(base_name)` - Retrieve unique table name
- `get_all_created_table_names()` - Get all table name mappings

**Functionality:**
1. **AWS SDK Setup**: Configures endpoint, region, test credentials
2. **Unique Table Naming**: Appends xid to base names (e.g., `users_ck7abc123`)
3. **Table Creation**: Converts `TableInfo` to `CreateTableInput` and creates tables
4. **Seed Data Loading**:
   - Reads JSON array from file
   - Converts to DynamoDB items via serde_dynamo
   - Batch writes in chunks of 25 items
5. **Automatic Cleanup** (with `test_utils` feature):
   - Implements `Drop` trait
   - Spawns background tokio threads to delete tables
   - Ensures cleanup even on panic

### Feature Gates

```toml
[features]
default = ["connector"]
connector = ["aws-config", "xid"]
test_utils = ["tokio"]
```

- **connector** (default): Main functionality with AWS integration and unique IDs
- **test_utils**: Adds automatic cleanup on drop (requires tokio runtime)

### Design Patterns

1. **Resource Acquisition Is Initialization (RAII)**: Automatic cleanup via Drop trait
2. **Type Safety**: Strong typing for DynamoDB attribute types
3. **Feature Gating**: Separate production and test utilities
4. **Configuration as Code**: YAML-based declarative schemas
5. **Builder Pattern**: Fluent API for constructing AWS SDK requests
6. **Error Propagation**: Result types with custom error enum
7. **Async/Await**: Full async support via tokio

## Build, Test, and Run Commands

### Prerequisites

1. **Install DynamoDB Local**:
```bash
# Download from AWS and extract
# Example location: ~/bin/dynamodb_local_latest/
```

2. **Start DynamoDB Local**:
```bash
java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib \
     -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar \
     -inMemory -sharedDb
```

Or use the GitHub Action in CI:
```yaml
- name: Setup DynamoDB Local
  uses: rrainn/dynamodb-action@v2.0.1
  with:
    port: 8000
    cors: '*'
```

### Build Commands

```bash
# Standard build
cargo build

# Build with all features
cargo build --all-features

# Release build
cargo build --release

# Via Makefile
make build
```

### Test Commands

```bash
# Run all tests with all features (requires DynamoDB Local running)
cargo test --all-features

# Run integration tests
cargo test --test connector_integration_test --all-features

# Via Makefile
make test

# Using nextest (faster test runner)
cargo nextest run --all-features

# With coverage
cargo llvm-cov nextest --all-features
```

**Note**: Integration tests require DynamoDB Local running on `localhost:8000`

### Linting & Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-features

# Clippy with warnings as errors
cargo clippy --all-features -- -D warnings

# Check compilation
cargo check --all-features
```

### Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

Hooks include:
- `cargo fmt` - Code formatting
- `cargo deny check` - Dependency auditing
- `typos` - Spell checking
- `cargo check` - Compilation check
- `cargo clippy` - Linting
- `cargo test` - Run tests

### Release Process

```bash
# Generate changelog and create release
make release

# This runs:
# 1. git cliff --tag <version> > CHANGELOG.md
# 2. git add CHANGELOG.md
# 3. git commit
# 4. git tag <version>
# 5. git push with tags
```

### Publishing

```bash
# Publish to crates.io
cargo publish
```

## Code Patterns and Conventions

### Error Handling

**Pattern**: Use `thiserror` for custom error types with context
```rust
#[derive(Debug, thiserror::Error)]
pub enum DynamoToolsError {
    #[error("Failed to read config from {file}: {source}")]
    ConfigRead {
        file: String,
        source: std::io::Error,
    },
    // ... more variants
}

pub type Result<T> = std::result::Result<T, DynamoToolsError>;
```

### Async/Await Usage

**Pattern**: All AWS SDK operations are async
```rust
pub async fn try_new(config: TableConfig) -> Result<Self> {
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&aws_config);
    // ... async operations
}
```

### Type Conversions

**Pattern**: Implement `From` and `TryFrom` traits for AWS SDK conversions
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

### Configuration Loading

**Pattern**: Support both file-based and programmatic configuration
```rust
// From file
let config = TableConfig::load_from_file("config.yml")?;

// Programmatic
let config = TableConfig::new(
    "us-east-1",
    "http://localhost:8000",
    vec![table_info],
    true
);
```

### Unique Naming

**Pattern**: Generate collision-free table names for parallel tests
```rust
let unique_id = xid::new().to_string();
let unique_name = format!("{}_{}", base_name, unique_id);
```

### Batch Operations

**Pattern**: Process items in AWS-compliant batches (max 25 items)
```rust
for chunk in items.chunks(25) {
    let write_requests: Vec<_> = chunk.iter()
        .map(|item| /* create WriteRequest */)
        .collect();

    client.batch_write_item()
        .request_items(table_name, write_requests)
        .send()
        .await?;
}
```

### Testing Patterns

**Pattern**: Use feature gates for test-specific functionality
```rust
#[cfg(feature = "test_utils")]
impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        if self.config.delete_on_exit {
            // Spawn background cleanup
        }
    }
}
```

**Pattern**: Integration tests with DynamoDB Local
```rust
#[cfg(all(test, feature = "test_utils"))]
#[tokio::test]
async fn test_name() -> anyhow::Result<()> {
    let connector = DynamodbConnector::load("fixtures/dev.yml").await?;
    // ... test operations
    Ok(())
}
```

### Naming Conventions

- **Modules**: Snake case (`connector.rs`, `error.rs`)
- **Types**: Pascal case (`TableConfig`, `DynamodbConnector`)
- **Functions/Methods**: Snake case (`load_from_file`, `get_created_table_name`)
- **Constants**: Screaming snake case (none currently defined)
- **Attributes**: Snake case (`table_name`, `delete_on_exit`)

### Documentation

**Pattern**: Use doc comments with examples
```rust
/// Creates a new DynamoDB connector from a configuration file.
///
/// # Arguments
/// * `file_path` - Path to YAML configuration file
///
/// # Example
/// ```no_run
/// let connector = DynamodbConnector::load("config.yml").await?;
/// ```
pub async fn load(file_path: &str) -> Result<Self>
```

### Clippy Configuration

```toml
[lints.clippy]
result_large_err = "allow"  # Allow large Result error types
```

## Version History

### v0.5.0 (2025-05-01) - Latest
- Added comprehensive error handling with thiserror
- Added integration tests
- **Breaking Change**: Multi-table support
- Seed data support
- Disabled test_utils feature by default

### v0.4.0 (2023-12-24)
- Upgraded AWS SDK to v1
- Fixed provisioning and billing mode handling

### v0.3.x (2022-2023)
- AWS SDK updates and endpoint configuration
- Added Debug/Clone for DynamodbConnector
- Support for TableInfo::load()

### v0.2.x (2022)
- Renamed from dynamodb-tester to dynamodb-tools
- Made table cleanup optional

### v0.1.x (2022)
- Initial release with basic functionality
- LSI and GSI support

## Dependencies & Security

### Dependency Management
- Uses `cargo-deny` for dependency auditing
- Regular updates to AWS SDK versions
- Pre-commit hooks check for issues

### Test Infrastructure
- DynamoDB Local for isolated testing
- GitHub Actions CI with automated testing
- No production AWS credentials needed

## Usage Example

```rust
use dynamodb_tools::{DynamodbConnector, TableConfig};
use aws_sdk_dynamodb::types::AttributeValue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let connector = DynamodbConnector::load("config.yml").await?;

    // Get unique table name
    let users_table = connector
        .get_created_table_name("users")
        .expect("users table created");

    // Use AWS SDK client
    let item = connector.client()?
        .get_item()
        .table_name(users_table)
        .key("user_id", AttributeValue::S("user_1".into()))
        .key("resource_type", AttributeValue::S("profile".into()))
        .send()
        .await?;

    if let Some(item) = item.item {
        println!("Item: {:?}", item);
    }

    Ok(())
    // Tables automatically cleaned up if test_utils feature enabled
}
```

## Notes

- This is a mature library focused on development/testing workflows
- Not intended for production DynamoDB deployments
- Excellent test coverage with integration tests
- Well-documented with examples in README
- Active maintenance with regular AWS SDK updates
