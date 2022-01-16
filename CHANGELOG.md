# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.17] - 2021-01-16

### Added

- Add hook `_init.sh` that runs everytime `scouty` starts. It helps to validate scripts

### Changed

- Fix parameter expansion for queued session argument in scripts

## [0.1.16] - 2021-01-15

### Changed

- Fix offline event

## [0.1.14] - 2021-01-15

### Added

- First public release
- Easily connect to westend, kusama or polkadot Parity public nodes
- Set optional matrix bot
- Hooks executed by on-chain events
- Runtimes (Polkadot/9140, Kusama/9150, Westend/9150)