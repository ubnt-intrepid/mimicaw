# Changelog

All notable changes to this project will be documented in this file.

This format is based on [Keep a Changelog], and this project adheres to [Semantic Versioning].

## [Unreleased]

## [0.1.3] (2020-02-02)

### Changed

* switch to `pin-project` ([6b7cc9e](https://github.com/ubnt-intrepid/mimicaw/commit/6b7cc9e3b83de5126f760e321dddbb2f9bfb492b))

## [0.1.2] (2020-01-27)

### Fixed

* replace inappropriate `Term::write_str` with `writeln!` ([d19fc98](https://github.com/ubnt-intrepid/mimicaw/commit/d19fc983def213eed5f7ed81e9991862125df2b9))

## [0.1.1] (2020-01-25)

### Added

* Support for reporting the test summary ([#2](https://github.com/ubnt-intrepid/mimicaw/pull/2)).
  The definition of `Report` introduced here is experimental and will be changed
  in the future version.

### Fixed

* Make sure to pin by the test case, rather than the collection of entire test cases (in [a5b372a](https://github.com/ubnt-intrepid/mimicaw/commit/a5b372a3d94fd606984579bd373f3688dec83b46) and [daa6526](https://github.com/ubnt-intrepid/mimicaw/commit/daa6526c5e483719944f3a298b805040bf368f32)).

## [0.1.0] (2020-01-24)

## [0.0.2] (2020-01-23)

## [0.0.1] (2020-01-23)

* initial release

<!-- links -->

[Unreleased]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.0.2...v0.1.0
[0.0.2]: https://github.com/ubnt-intrepid/mimicaw/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/ubnt-intrepid/mimicaw/tree/v0.0.1

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
