# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [0.5.0](https://github.com/tyrchen/dynamodb-tools/compare/v0.4.0..v0.5.0) - 2025-05-01

### Features

- add thiserror - ([51837c5](https://github.com/tyrchen/dynamodb-tools/commit/51837c51171c138279998805795cace8a75731b7)) - Tyr Chen
- add integration test - ([2ae8438](https://github.com/tyrchen/dynamodb-tools/commit/2ae8438383b0a419268f821f8f70536869faf5ac)) - Tyr Chen
- add extra integration tests - ([0f0ff9c](https://github.com/tyrchen/dynamodb-tools/commit/0f0ff9c500370fdfde9e5daa14d485d29a8cead2)) - Tyr Chen
- add doc test - ([8257262](https://github.com/tyrchen/dynamodb-tools/commit/825726284622f69d619b0fc17daa74c9599e8ebe)) - Tyr Chen
- support multi table - ([8c8e603](https://github.com/tyrchen/dynamodb-tools/commit/8c8e603d351bce82578f8591a0a81a2988bc9181)) - Tyr Chen
- support seed data - ([f58290b](https://github.com/tyrchen/dynamodb-tools/commit/f58290b03e628645d9ce93ca329fd995b821d51d)) - Tyr Chen

### Miscellaneous Chores

- bump version for deps - ([fb8fbd6](https://github.com/tyrchen/dynamodb-tools/commit/fb8fbd6dca10e738f2852a7603da87c951e74df8)) - Tyr Chen
- add cursor rules - ([8cc6298](https://github.com/tyrchen/dynamodb-tools/commit/8cc62988fde7e276383c281462dc81a1c7c7123a)) - Tyr Chen
- initialize memory bank - ([b1b600a](https://github.com/tyrchen/dynamodb-tools/commit/b1b600a9da72dbaf285cf4fe81a5654cbcf3052c)) - Tyr Chen
- disable test_utils feature - ([5048f12](https://github.com/tyrchen/dynamodb-tools/commit/5048f129aff7c7f52881f9c0abbb53b892d4b943)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([f189441](https://github.com/tyrchen/dynamodb-tools/commit/f1894414cbfa5820e54da64c8615fff548e3561d)) - Tyr Chen
- Merge pull request #1 from tyrchen/feature/upgrade

Update based on AI's suggestions. Supported:

more unit test cases
add integration test
support multiple table.
Note: this is a breaking change. - ([179b12f](https://github.com/tyrchen/dynamodb-tools/commit/179b12f17e58d07bdc5772aa2562dccdff31ce21)) - Tyr Chen

---
## [0.4.0](https://github.com/tyrchen/dynamodb-tools/compare/v0.3.5..v0.4.0) - 2023-12-24

### Bug Fixes

- fix provision & billing mode - ([791e5b1](https://github.com/tyrchen/dynamodb-tools/commit/791e5b18cb7b7228ab12b4e4158eee52d025d7cb)) - Tyr Chen

### Features

- upgrade aws sdk - ([dc0ed1a](https://github.com/tyrchen/dynamodb-tools/commit/dc0ed1ae29a61bd1c290b5a4dfbffcbefc18801f)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([0b585ad](https://github.com/tyrchen/dynamodb-tools/commit/0b585adc71ebbfb94a6194354d52263a3ea11c82)) - Tyr Chen

---
## [0.3.5](https://github.com/tyrchen/dynamodb-tools/compare/v0.3.4..v0.3.5) - 2023-02-03

### Miscellaneous Chores

- bump dynamodb version - ([a54ffe0](https://github.com/tyrchen/dynamodb-tools/commit/a54ffe0919bb00cfd1613569b717c2bbe21c0ff9)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([6381a32](https://github.com/tyrchen/dynamodb-tools/commit/6381a32c874877128eb57c6db7b660dc966e4f6a)) - Tyr Chen

---
## [0.3.4](https://github.com/tyrchen/dynamodb-tools/compare/v0.3.3..v0.3.4) - 2023-01-20

### Miscellaneous Chores

- upgrade aws sdk and fix endpoint deprecation issue - ([73ed694](https://github.com/tyrchen/dynamodb-tools/commit/73ed694daf8d04920a83eb3997ccc5655d9cccfa)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([7457ef3](https://github.com/tyrchen/dynamodb-tools/commit/7457ef303aed615bf5def50d6e12cacdb45a3e01)) - Tyr Chen

---
## [0.3.3](https://github.com/tyrchen/dynamodb-tools/compare/v0.3.1..v0.3.3) - 2023-01-02

### Features

- upgrade aws deps to latest version (not ready to release yet since the ecosystem hasn't picked 0.52 up yet) - ([c20247e](https://github.com/tyrchen/dynamodb-tools/commit/c20247e6b21b5a8a68fda1446b88a129531af77f)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([3c1dd02](https://github.com/tyrchen/dynamodb-tools/commit/3c1dd02cd511bb0f8e285d7c23107b86bc6512d3)) - Tyr Chen
- upgrade deps - ([9ee055d](https://github.com/tyrchen/dynamodb-tools/commit/9ee055ddf4540040f0e43320c0f423bfea0e91ed)) - Tyr Chen

---
## [0.3.1](https://github.com/tyrchen/dynamodb-tools/compare/v0.3.0..v0.3.1) - 2022-12-12

### Miscellaneous Chores

- add Debug/Clone for DynamodbConnector, support load for TableInfo - ([69e41ee](https://github.com/tyrchen/dynamodb-tools/commit/69e41ee4a3ce8ef6408f0dfc8cd5a14a75c36fe8)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([50fe2ff](https://github.com/tyrchen/dynamodb-tools/commit/50fe2ff85611c4668e3897820d0e562a76abc59c)) - Tyr Chen

---
## [0.3.0](https://github.com/tyrchen/dynamodb-tools/compare/v0.2.2..v0.3.0) - 2022-12-12

### Refactoring

- repurpose the local tester to connector - ([0855106](https://github.com/tyrchen/dynamodb-tools/commit/0855106aafc0ba8fe025dbdb726cbdd5cd7b47b9)) - Tyr Chen

---
## [0.2.2](https://github.com/tyrchen/dynamodb-tools/compare/v0.2.1..v0.2.2) - 2022-12-11

### Other

- Update CHANGELOG.md - ([873daca](https://github.com/tyrchen/dynamodb-tools/commit/873daca4ba12e20fa65348e88464ed6bd68c5c9b)) - Tyr Chen
- deprecate dynamodb-tester and change its name to dynamodb-tools - ([7caa3ec](https://github.com/tyrchen/dynamodb-tools/commit/7caa3ecb80942c68e4e98620ea208ce5a2489190)) - Tyr Chen

---
## [0.2.1](https://github.com/tyrchen/dynamodb-tools/compare/v0.2.0..v0.2.1) - 2022-12-11

### Features

- allow drop table to be optional - ([41af7f2](https://github.com/tyrchen/dynamodb-tools/commit/41af7f23951fd1a335523961805b301f76447f25)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([25d84a5](https://github.com/tyrchen/dynamodb-tools/commit/25d84a51d361e5820f96ccb84a3663bec7a2d5d1)) - Tyr Chen

---
## [0.2.0](https://github.com/tyrchen/dynamodb-tools/compare/v0.1.3..v0.2.0) - 2022-12-11

### Other

- Update CHANGELOG.md - ([5b4b250](https://github.com/tyrchen/dynamodb-tools/commit/5b4b2509d38e077102aaf3acb0207407dc113906)) - Tyr Chen

### Refactoring

- move most of the code to local feature and provide DynamoClient trait - ([4ca018e](https://github.com/tyrchen/dynamodb-tools/commit/4ca018ea460b5e803c2ce8eb70fec2903abd5227)) - Tyr Chen

---
## [0.1.3](https://github.com/tyrchen/dynamodb-tools/compare/v0.1.2..v0.1.3) - 2022-12-11

### Bug Fixes

- lsi sk should be mandatory - ([7949054](https://github.com/tyrchen/dynamodb-tools/commit/7949054a9b7f57f12489565d6185a4001f862b1e)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([75e9b00](https://github.com/tyrchen/dynamodb-tools/commit/75e9b0003f625f47251cb66551d7ffb60d99024f)) - Tyr Chen

---
## [0.1.2](https://github.com/tyrchen/dynamodb-tools/compare/v0.1.1..v0.1.2) - 2022-12-11

### Bug Fixes

- if attrs is empty, use all for lsi projection - ([fdf5044](https://github.com/tyrchen/dynamodb-tools/commit/fdf5044a3d0812fc4eb4290b90f084173ab179cb)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([66447af](https://github.com/tyrchen/dynamodb-tools/commit/66447afb982af19dadff9ca4b25a08c0ab6076f9)) - Tyr Chen

---
## [0.1.1] - 2022-12-11

### Bug Fixes

- fix gh action - ([44a31be](https://github.com/tyrchen/dynamodb-tools/commit/44a31be6824a5b0e4ed430625f489caa2363c804)) - Tyr Chen
- export TableConfig - ([60d04cc](https://github.com/tyrchen/dynamodb-tools/commit/60d04cca0eee440f3b90e76b9dca1a2ab5c52fc5)) - Tyr Chen

### Features

- support basic functionality of dynamodb tester - ([792c141](https://github.com/tyrchen/dynamodb-tools/commit/792c141c4555b53e47ef58786ee5b98557cfe92a)) - Tyr Chen

<!-- generated by git-cliff -->
