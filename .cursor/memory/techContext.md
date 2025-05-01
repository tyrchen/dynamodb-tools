# Technical Context: dynamodb-tools

## Language & Runtime

*   **Language:** Rust (Edition 2024)
*   **Async Runtime:** Tokio

## Build & Dependencies

*   **Build System:** Cargo
*   **Key Dependencies:**
    *   `aws-sdk-dynamodb`: Official AWS SDK for DynamoDB.
    *   `aws-config`: For loading AWS credentials and configuration.
    *   `anyhow`: For error handling.
    *   `serde` / `serde_yaml`: For configuration file parsing.
    *   `xid`: For generating unique IDs (likely for table names).
    *   `tokio`: For async runtime and testing.

## Development Environment

*   **DynamoDB Local:** Required for local development and testing. Needs to be run separately.
*   **Testing:** Unit and integration tests likely run via `cargo test`.
*   **Linting/Formatting:** Likely uses `rustfmt` and `clippy` (implied by standard Rust practices).
*   **CI:** GitHub Actions (`.github/workflows/build.yml`).
