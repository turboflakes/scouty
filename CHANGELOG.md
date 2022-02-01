# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2021-01-31

### Changed

- Rename block height to block grift in examples script
- Fix APR value to be displayed as percentage

## [0.2.1] - 2021-01-31

### Changed

- Fix Project APR missing from `_new_session` and `_new_era` hooks

## [0.2.0] - 2021-01-31

### Added

- Add block number to session and era scripts has positional argument `${9}`
- Expose Para validator under a new flag `--expose-para-validator`. If para validator in current session is available through positional argument `${22}` and the total number of para validator times in previous 6 sessionds in argument `${23}`
- Expose Era points under a new flag `--expose-era-points` with previous validator era points available at `${24}` and average points at position `${25}`
- Projected APR calculation based on Polkadot.js. Value available at position `${13}`
- Additional system metrics loaded directly from the server

### Changed

- Review `_init.sh`, `_new_era.sh`, `_new_session.sh` examples. 
- Adjusted 1KV nominators message
- Breaking change: With the addition of the APR at position `${13}` all next arguments have shift their position by +1.

## [0.1.25] - 2021-01-24

### Changed

- Fix parity default endpoints

## [0.1.24] - 2021-01-24

### Added

- Expose Total Nominators under a new flag `--expose-total-nominators`
- Add new flag `--expose-all` to expose all positional arguments in scripts using only a single flag

### Changed

- Review sripts to fit total nominators and give a better readability
- Update metadata Polkadot runtime/9151

## [0.1.23] - 2021-01-22

### Added

- Total authored blocks from previous 6 sessions

### Changed

- Fix Authored blocks

## [0.1.22] - 2021-01-18

### Changed

- Fix TBD argument at new session and new era hook scripts

## [0.1.21] - 2021-01-18

### Changed

- Update metadata Kusama runtime/9151

## [0.1.20] - 2021-01-18

### Changed

- Fix default values for optional flags. This makes it less error prone when working with scripts arguments.

## [0.1.19] - 2021-01-18

### Added 

- Add flag `--expose-authored-blocks` which exposes the number of blocks authored by the predefined stashes

## [0.1.18] - 2021-01-17

### Added 

- Add flag `--expose-network` which exposes network properties to all hooks

### Changed

- expose total stake, own stake, nominators and nominators stake to some of the hooks
- update positional arguments in the default default scripts

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