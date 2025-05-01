# System Patterns: dynamodb-tools

## Architecture

*   **Library Crate:** Designed to be included as a dependency in other Rust projects.
*   **Configuration-Driven:** Uses a YAML file (`config.yml`) for DynamoDB connection details and table definitions.
*   **RAII for Cleanup:** Leverages Rust's `Drop` trait on the `DynamodbConnector` to ensure resources (tables) are cleaned up automatically when the connector goes out of scope.

## Key Design Patterns

*   **Connector Pattern:** Provides a single entry point (`DynamodbConnector`) to manage the interaction with DynamoDB Local.
*   **Builder Pattern (implied):** The AWS SDK likely uses builder patterns for constructing requests (e.g., `put_item()`).
*   **Asynchronous Programming:** Uses `async/await` (tokio runtime) for non-blocking I/O operations with DynamoDB.
