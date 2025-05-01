# Product Context: dynamodb-tools

## Target Audience

*   Rust developers building applications that interact with AWS DynamoDB.
*   Teams looking for reliable ways to test DynamoDB interactions locally.

## High-Level Features

*   DynamoDB Local client management.
*   Configuration loading from YAML (`fixtures/config.yml`).
*   Automatic, unique table name generation.
*   Resource cleanup via RAII (Drop trait).
*   Integration examples for local development and CI (GitHub Actions).
