# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2019-06-29

### Added
- Help screen to tell users how to use the program

### Fixed
- Bug #16: relative paths were generated incorrectly before
- Bug #13: on mixed tag state, do nothing instead of the old dangerous
  and unintuitive behaviour

## [0.1.0] - 2019-06-28

First usable version.

### Added
- Terminal UI based on [Cursive](https://github.com/gyscos/cursive).
  - Shows side-by-side views of items and tags
  - Scrollbars when things don't fit
- Specify path to items directory and tags directory
- Select/deselect items to manage many at once
- Clearly show tag states for the selected items
- Open selected items with a specified command
- Symlinks use correct relative paths
- Create new tags
