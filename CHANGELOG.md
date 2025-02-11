<!--
    When adding new changelog entries, use [Issue #0] to link to issues and
    [PR #0 @user] to link to pull requests. Then run:

        cargo xtask update-changelog

    to update the actual links at the bottom of the file.
-->

### Version 2.0.3

* Upgrade xz version in precompiled binaries ([Issue #138], [PR #139 @chenxiaolong])
    * This fixes the `ota extract` and `ota verify` commands in some multithreaded situations.
* Add `--version` option to print out avbroot's version ([Issue #138], [PR #140 @chenxiaolong])

### Version 2.0.2

* Fix `data_offset` being set for payload operations that don't need it ([PR #136 @chenxiaolong])
    * This fixes patched stock OnePlus images from being rejected when flashing.

Behind-the-scenes changes:

* Move full OTA check to CLI functions to allow library functions to parse delta OTAs ([PR #135 @chenxiaolong])
* Remove unnecessary use of `anyhow` macro ([PR #137 @chenxiaolong])

### Version 2.0.1

* Add support for Magisk 263xx ([PR #132 @chenxiaolong])

### Version 2.0.0

* Initial Rust release. The old Python implementation can be found in the `python` branch. ([PR #130 @chenxiaolong])

<!-- Do not manually edit the lines below. Use `cargo xtask update-changelog` to regenerate. -->
[Issue #138]: https://github.com/chenxiaolong/avbroot/issues/138
[PR #130 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/130
[PR #132 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/132
[PR #135 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/135
[PR #136 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/136
[PR #137 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/137
[PR #139 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/139
[PR #140 @chenxiaolong]: https://github.com/chenxiaolong/avbroot/pull/140
