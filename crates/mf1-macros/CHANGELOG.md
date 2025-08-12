# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).



## [0.1.8](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.7...mf1-macros-v0.1.8) - 2025-08-12

### Documentation

- Add comments explaining function pointer comparisons in [`cc0745c`](https://github.com/JadedBlueEyes/messageformat/commit/cc0745ce9c26246b7b17a51b6c5b3c399e778cfb)

### Refactor

- Use more efficient iterators in [`7389a62`](https://github.com/JadedBlueEyes/messageformat/commit/7389a6272ac3c1f748a31edb6291ec601111957b)



## [0.1.7](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.6...mf1-macros-v0.1.7) - 2025-05-11

### Miscellaneous Tasks

- Update breaking dep versions in [`62030c9`](https://github.com/JadedBlueEyes/messageformat/commit/62030c922e62fcf7fa7ea2c4348312fb7e253568)
- More explicitly transform to proc_macro2 tokens in [`bb14afe`](https://github.com/JadedBlueEyes/messageformat/commit/bb14afe1b0ea35e0c68739fc47051d658bfac826)
- Fix typos in [`6fe2c13`](https://github.com/JadedBlueEyes/messageformat/commit/6fe2c13120f6ed2d254778e0422a3960ed4d2ea0)



## [0.1.6](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.5...mf1-macros-v0.1.6) - 2024-09-13

### Miscellaneous Tasks

- Enrich Cargo metadata in [`25fe330`](https://github.com/JadedBlueEyes/messageformat/commit/25fe330d5351a1dc9549af51abf49afcf85199fb)



## [0.1.5](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.4...mf1-macros-v0.1.5) - 2024-08-22

### Bug Fixes

- Allow multiple interpolation values in [`3228ee1`](https://github.com/JadedBlueEyes/messageformat/commit/3228ee11e3a78cc407552ca6f43e5e96b38e1a9b)

## [0.1.4](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.3...mf1-macros-v0.1.4) - 2024-08-02

### Documentation

- Remove duplicate link refs in [`f754fc8`](https://github.com/JadedBlueEyes/messageformat/commit/f754fc8dd33df5b415a7f8af089be0025390fd3c)
- Clean up changelogs in [`110b4ac`](https://github.com/JadedBlueEyes/messageformat/commit/110b4ac49c8fd73aeb9e119950e44c3edb2c00a4)

### Features

- Add a constant listing all available locales in [`24d5c7e`](https://github.com/JadedBlueEyes/messageformat/commit/24d5c7e861196b0b0d4cb53c70897e8510bf199f)

## [0.1.3](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.2...mf1-macros-v0.1.3) - 2024-07-22

### Documentation

- Add more badges to README files in [`77c587b`](https://github.com/JadedBlueEyes/messageformat/commit/77c587b5222b26032dfa40eb8777cf0af3f9a32f)

### Features

- Add support for subkeys in [`eb65424`](https://github.com/JadedBlueEyes/messageformat/commit/eb65424120fd80964057950b95975546265962f6)

### Miscellaneous Tasks

- Add `repository` to Cargo.toml files in [`f08a90a`](https://github.com/JadedBlueEyes/messageformat/commit/f08a90a8f25cb89d5c1996d992fabec191eda186)

### Refactor

- Start implementing subkeys in [`a040e7e`](https://github.com/JadedBlueEyes/messageformat/commit/a040e7ea88ce34d328b1f3d82ef488c8c8738ec9)

## [0.1.2](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.1...mf1-macros-v0.1.2)  - 2024-07-12

### Documentation

- Add bare-bones README files ([6861793](https://github.com/JadedBlueEyes/messageformat/commit/6861793fe974f384a2136ee1550eba9fbf592796))

### Miscellaneous Tasks

- Specify readme files in Cargo.toml ([21c51b9](https://github.com/JadedBlueEyes/messageformat/commit/21c51b9038d9b74a8cd13b75237f20b1ed11c8c4))

## [0.1.1](https://github.com/JadedBlueEyes/messageformat/compare/mf1-macros-v0.1.0...mf1-macros-v0.1.1) - 2024-07-12

### Other
- Fill out changelogs and configure git-cliff

## [0.1.0] - 2024-07-12

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
