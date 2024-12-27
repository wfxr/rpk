
## [0.2.1-rc.4](https://github.com/wfxr/rpk/compare/v0.2.0..0.2.1-rc.4) (2024-12-27)

### 🚀 Features

- Improve binary selection logic - ([c60c3b5](https://github.com/wfxr/rpk/commit/c60c3b5dac04af51df5ee5de4a4923ff2ebc84a9))
- Fix log level - ([1d64d15](https://github.com/wfxr/rpk/commit/1d64d15b6830306becc3364a454efbd62bb1adc8))
- Compatibility with old lock files - ([a291e2b](https://github.com/wfxr/rpk/commit/a291e2ba41371bdf73933391ffe4a0d750a2602d))
- Support multiple binaries in a package - ([bb89c72](https://github.com/wfxr/rpk/commit/bb89c724ca390d3538790c055b33f75fa010a31c))
- Improve asset selection logic - ([99567aa](https://github.com/wfxr/rpk/commit/99567aa4ea5bdcd77dfd39f27d28ff69d1bc0536))

### 🚜 Refactor

- Improve error handling misc - ([f7f7f5d](https://github.com/wfxr/rpk/commit/f7f7f5d81c4feaac64273b0ac745a0d25fdc69e4))
- Update subcommand names - ([18eae36](https://github.com/wfxr/rpk/commit/18eae36b0b30de46ccb622c1ed2458f986ad33aa))

### 📚 Documentation

- Update README - ([e69bf5e](https://github.com/wfxr/rpk/commit/e69bf5ef233b6cd139638e4f96c8adbd55b54dbd))

### ⚙️ Miscellaneous Tasks

- Update macos-13 to macos-14 - ([80c5387](https://github.com/wfxr/rpk/commit/80c53871fcf551461a48c7cc52daa605805e3043))
- Bump up dependencies - ([5e5c19c](https://github.com/wfxr/rpk/commit/5e5c19ceab4237b5153791a1f2890fe063e7b70c))
- Update rust toolchain to nightly-2024-08-20 - ([4fa6fe7](https://github.com/wfxr/rpk/commit/4fa6fe7d4f4fe2692b0916186ff7c6283bf61294))

## [0.2.0](https://github.com/wfxr/rpk/compare/v0.1.8..v0.2.0) (2024-08-20)

### 🚀 Features

- Add socks support for ureq - ([d3661aa](https://github.com/wfxr/rpk/commit/d3661aad4f014155f7bd8f14df34c40503856728))
- Add cleanup command & improve completions command - ([6ab4fc6](https://github.com/wfxr/rpk/commit/6ab4fc6e19711d4dc16df78edcaf1e9776d68d0c))
- Improve the `env` output - ([b34cf00](https://github.com/wfxr/rpk/commit/b34cf006e0b4369dab7e4225d414b59345fbbb28))
- Lock config dir when running commands - ([0dd3270](https://github.com/wfxr/rpk/commit/0dd327033fbae268c1feb4defc9a9712deab7210))
- Support parallel update - ([92d6692](https://github.com/wfxr/rpk/commit/92d6692d41fd7374d6fb55af9f5220728b729787))
- Support parallel & remove progress bar - ([153b722](https://github.com/wfxr/rpk/commit/153b722f9d73663cd35a8628ae050ddccbea4c0a))
- Emojify package descriptions when doing search - ([ebb6983](https://github.com/wfxr/rpk/commit/ebb6983fd69608b30dfd2db848c7c6b57724444e))
- Replace async api with blocking api - ([9c0d825](https://github.com/wfxr/rpk/commit/9c0d825eb9c8a3c962c1fd6cb3e95dbc4faf5878))
- Replace curl with ureq for http requests - ([c4cebc0](https://github.com/wfxr/rpk/commit/c4cebc089221c90933b12f5b6e16ec7669d34730))
- Replace reqwest with curl - ([d6540d7](https://github.com/wfxr/rpk/commit/d6540d7e5afbdf28fea536106c6c305524ae1f81))
- Render emojis in package description - ([eb46b6a](https://github.com/wfxr/rpk/commit/eb46b6a547e769730039fa81ce9d4ff855d237b2))
- Add `ls` and `fd` aliases for commands - ([a6e5379](https://github.com/wfxr/rpk/commit/a6e5379c8812f8e0389acb535e26f59e0f3e85c7))

### 🐛 Bug Fixes

- Sync should always generate a new locked package list - ([523a7ba](https://github.com/wfxr/rpk/commit/523a7bac6adb1e0cece5bd2a94f80ef19258f74d))

### 🚜 Refactor

- Add Shorten trait for path shortening - ([a06ff06](https://github.com/wfxr/rpk/commit/a06ff06cea31b1a091d4f09e97a2a126bfc77868))
- Validate repo using clap's value_parser - ([70c6ea8](https://github.com/wfxr/rpk/commit/70c6ea8ac6dec38abf20d80c469b14b4eb0953c7))
- Improve error handling in filter_assets - ([ffd17c9](https://github.com/wfxr/rpk/commit/ffd17c94c107e117b6cd6208e8ff11de9d7f6346))
- Change package install directory layout - ([dac6b2a](https://github.com/wfxr/rpk/commit/dac6b2a3b2225cf2266a06c90f1399bd34760187))

### 📚 Documentation

- Update README.md and add a config sample - ([ad16d40](https://github.com/wfxr/rpk/commit/ad16d40692d9e9cb0caba4fe71f41bfaa7aeddb1))

## [0.1.8](https://github.com/wfxr/rpk/compare/v0.1.7..v0.1.8) (2024-08-12)

### 🚀 Features

- Add list command - ([b5f3cc1](https://github.com/wfxr/rpk/commit/b5f3cc1e34103b0bb1e7dac06b29f6d8cd6390cb))

### 🚜 Refactor

- Skip serializing default source tag - ([84f8b02](https://github.com/wfxr/rpk/commit/84f8b02b59a80f07a6205f78a7513722f8288953))

### ⚙️ Miscellaneous Tasks

- Ignore release commit message in cliff.toml - ([18f159c](https://github.com/wfxr/rpk/commit/18f159ca476703748b79411ee49c513d724dbb85))
- Bump up dependencies - ([67e795e](https://github.com/wfxr/rpk/commit/67e795e44b901e37a85c9dbbcbd0e7d74c8a7e50))

## [0.1.7](https://github.com/wfxr/rpk/compare/v0.1.6..v0.1.7) (2024-08-11)

### 🐛 Bug Fixes

- Add .apk, .msi to the list of ignored asset extensions - ([0df133b](https://github.com/wfxr/rpk/commit/0df133b58170b6427b5b964290cad97d2a998d90))

### 🚜 Refactor

- Fix shellcheck warning in scripts/install - ([6e1ae65](https://github.com/wfxr/rpk/commit/6e1ae6594eda8d6949ef1a1bb37208023b93777d))

## [0.1.5](https://github.com/wfxr/rpk/compare/v0.1.4..v0.1.5) (2024-08-11)

### 🚀 Features

- Support to do init during installation - ([f112f7f](https://github.com/wfxr/rpk/commit/f112f7f8d573ce30b9c2e5411c29e187e8881318))
- Remove lock file when initializing config - ([0a17179](https://github.com/wfxr/rpk/commit/0a17179a956ded5138c7c713813e8d9f707f9df4))
- Set rpk as the default package in template - ([6bdef0f](https://github.com/wfxr/rpk/commit/6bdef0fe5d483b6eae4af0a33534462ec9c32886))

### 🚜 Refactor

- Add log output for init command - ([14a3092](https://github.com/wfxr/rpk/commit/14a3092ce2a2f9aa4cc1d6b4c6a3753833af9166))

## [0.1.4](https://github.com/wfxr/rpk/compare/v0.1.3..v0.1.4) (2024-08-11)

### 🚀 Features

- Add `rpk init` command - ([c3b51f9](https://github.com/wfxr/rpk/commit/c3b51f988c5e3a9c4a88542185601f6837c2452e))
- Rename search to find & add command aliases - ([11854fd](https://github.com/wfxr/rpk/commit/11854fdf687b03d65a6bab5e055386343b9e7c91))

## [0.1.3](https://github.com/wfxr/rpk/compare/v0.1.2..v0.1.3) (2024-08-11)

### 🚀 Features

- Change progress bar characters - ([a3b8bb4](https://github.com/wfxr/rpk/commit/a3b8bb4f51bb84bca85283689f834d5652b11a5e))
- Improve installer & downloader - ([f7a1726](https://github.com/wfxr/rpk/commit/f7a17263424b39d1356db7b0c785f938e4ec4cb0))
- Improve assert downloader - ([c6032d9](https://github.com/wfxr/rpk/commit/c6032d9d4da4d065cd2fd7ae9153fa19e4fd972a))

### ⚙️ Miscellaneous Tasks

- Add pre-commit config - ([5eb359f](https://github.com/wfxr/rpk/commit/5eb359f65cd92bc312f2ffb4867b79a5c28bbb10))

## [0.1.2](https://github.com/wfxr/rpk/compare/v0.1.1..v0.1.2) (2024-08-10)

### 🚀 Features

- Allow user to input package name - ([77385b2](https://github.com/wfxr/rpk/commit/77385b2023a8eb118d08ed99ba28076b841153c9))
- Enchance the binary installation - ([ea28a14](https://github.com/wfxr/rpk/commit/ea28a14c49c55f110e2be96e4b2477754d9674b3))
- Handle i686 arch in github provider - ([4ff4532](https://github.com/wfxr/rpk/commit/4ff45328e44f4bfb2213fad3dcbf85c1d149fa3e))
- Handle arm architecture in github provider - ([2971de4](https://github.com/wfxr/rpk/commit/2971de470d7cf289c3219a1ab926ecf22efacc3d))

### 🐛 Bug Fixes

- Some packages have no common prefix in the archive - ([962bff1](https://github.com/wfxr/rpk/commit/962bff16cd7176c395bb0d0264149cfc9d923efb))

## [0.1.1] - 2024-08-09

### ⚙️ Miscellaneous Tasks

- Prefer musl libc over glibc - ([a781c0e](https://github.com/wfxr/rpk/commit/a781c0e52d9c6b3414cedcefabe6e30298066429))
- Port install script from rossmacarthur/install - ([af96e03](https://github.com/wfxr/rpk/commit/af96e03047cb2af0f9801948cfdcbdd0984fd9b4))

### Misc

- Add .gitattributes to ignore completions/ dir - ([2f17136](https://github.com/wfxr/rpk/commit/2f171361771dee3a4af02cb211bdef09d00e8757))

<!-- generated by git-cliff -->
