# Task List

## Active Tasks

*   [x] **VAN:** Complete initial project setup and analysis.
*   [ ] **PLAN:** Review code, identify improvements/features, and update Memory Bank.

## Planned Tasks (Resulting from PLAN)

### Improvements
*   [ ] **Refactor Error Handling:** Replace some `anyhow`/`.expect()` with custom errors (`thiserror`) in `config.rs` and `connector.rs` for more specific feedback.
*   [ ] **Improve Drop Implementation:** Refactor the `Drop` trait in `connector.rs` to handle asynchronous cleanup more robustly (avoid `block_on` in new thread if possible) and improve error reporting (use logging?).
*   [ ] **Clarify `delete_on_exit`:** Make the behavior and configuration of `delete_on_exit` consistent between test and non-test builds.
*   [ ] **Enhance Testing:** Move integration tests to `tests/` directory and add more comprehensive test cases covering schema variations and error conditions.
*   [ ] **Improve Documentation:** Add detailed rustdoc comments to public items in `lib.rs`, `config.rs`, and `connector.rs`.

### New Features
*   [ ] **Add Multiple Table Support:** Modify `TableConfig` and `DynamodbConnector` to handle definitions and lifecycle for multiple tables.
*   [ ] **Implement Data Seeding:** Add functionality to load initial data into created tables based on config.
*   [ ] **Allow Explicit AWS Config:** Extend `TableConfig` to include optional fields for AWS region, endpoint, and credentials.

## Future Tasks (Potential)

*   [ ] **CREATIVE:** Design solutions for planned tasks (if needed, unlikely for these).
*   [ ] **IMPLEMENT:** Implement planned improvements and features.
*   [ ] **QA:** Test implemented features and improvements.
