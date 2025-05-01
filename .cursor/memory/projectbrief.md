# Project Brief: dynamodb-tools

## Core Purpose

`dynamodb-tools` is a Rust crate designed to simplify the integration and testing of applications against AWS DynamoDB Local. It automates the setup and teardown of temporary DynamoDB tables for isolated testing environments.

## Key Goals

*   Provide an easy-to-use connector (`DynamodbConnector`) for managing DynamoDB Local instances.
*   Automatically create uniquely named tables for tests.
*   Ensure automatic cleanup of test tables upon completion (using `Drop` trait).
*   Facilitate integration with CI/CD pipelines (e.g., GitHub Actions).
