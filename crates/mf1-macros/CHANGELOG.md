# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.2...mf1-macros-v0.1.3) - 2024-07-22

### Documentation

- Add more badges to README files in [`77c587b`](https://github.com/JadedBlueEyes/messageformat/commit/77c587b5222b26032dfa40eb8777cf0af3f9a32f)

### Features

- Add support for subkeys in [`eb65424`](https://github.com/JadedBlueEyes/messageformat/commit/eb65424120fd80964057950b95975546265962f6)

### Miscellaneous Tasks

- Add `repository` to Cargo.toml files in [`f08a90a`](https://github.com/JadedBlueEyes/messageformat/commit/f08a90a8f25cb89d5c1996d992fabec191eda186)

### Refactor

- Start implementing subkeys in [`a040e7e`](https://github.com/JadedBlueEyes/messageformat/commit/a040e7ea88ce34d328b1f3d82ef488c8c8738ec9)
[0.1.3]: https://github.com/JadedBlueEyes/messageformat/compare/0.1.2..0.1.3

## [0.1.2] - 2024-07-12

### Documentation

- Add bare-bones README files ([6861793](https://github.com/JadedBlueEyes/messageformat/commit/6861793fe974f384a2136ee1550eba9fbf592796))

### Miscellaneous Tasks

- Specify readme files in Cargo.toml ([21c51b9](https://github.com/JadedBlueEyes/messageformat/commit/21c51b9038d9b74a8cd13b75237f20b1ed11c8c4))

## [0.1.1](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.0...mf1-macros-v0.1.1) - 2024-07-12

### Other
- Fill out changelogs and configure git-cliff

## [mf1-macros-v0.1.0] - 2024-07-12

### Bug Fixes

- Properly qualify FromStr ([e63c999](https://github.com/JadedBlueEyes/messageformat/commit/e63c999a149761d8c4b0aea46bfba977e516e588))
- Properly substitute return type for builder ([b142352](https://github.com/JadedBlueEyes/messageformat/commit/b1423525f4ead5674d1205b921aea3b0a41740b3))
- Use HashSet to avoid bogus warning ([4f3029d](https://github.com/JadedBlueEyes/messageformat/commit/4f3029d35104b389b06bf0628463bf2770bc290f))

### Documentation

- Add CHANGELOG files ([7fec2dd](https://github.com/JadedBlueEyes/messageformat/commit/7fec2ddb40381df682d1dd6fde88375b5b209ef0))
- Add descriptions to crates ([1c2c01e](https://github.com/JadedBlueEyes/messageformat/commit/1c2c01ebce34881b18a28f249c506b8f2950c6f2))

### Features

- Initial  macro implementation ([5a85913](https://github.com/JadedBlueEyes/messageformat/commit/5a8591366b5b521a454d9152bbdb1534ba3415ac))
- Parse loaded locale strings in load_locales! macro ([b27f062](https://github.com/JadedBlueEyes/messageformat/commit/b27f0623b8e502b8aae598ea0f3d8a5763ce7404))
- Add initial t! macro ([4c3dc37](https://github.com/JadedBlueEyes/messageformat/commit/4c3dc37a3092188d7828ff716da4f914f0080b25))
- Add interpolation runtime support ([9412234](https://github.com/JadedBlueEyes/messageformat/commit/941223468282210ee239ccfef496f6908e74c19e))
- Add support for SelectFormat at runtime ([ab29742](https://github.com/JadedBlueEyes/messageformat/commit/ab29742c8a8c8df3f539e4e09e12f30610161411))

### Miscellaneous Tasks

- Add licences ([954312a](https://github.com/JadedBlueEyes/messageformat/commit/954312ad5ed23d4e9a2415f9ddac822f8ed24f60))

### Refactor

- Make locale string collections static structs ([cbeccf2](https://github.com/JadedBlueEyes/messageformat/commit/cbeccf23052ca79757185a94542b07dff1ab60d2))
- Embed locale strings as const rather than static ([71f69a7](https://github.com/JadedBlueEyes/messageformat/commit/71f69a7fbd59da7b7f38d869f848ceafe2705646))
