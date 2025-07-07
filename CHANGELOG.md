# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/advantic-au/mqi/compare/v0.1.0...v0.2.0) - 2025-07-06

### Added

- Added "mqc_latest" feature for latest MQ version dependency
- Removed re pub of libmqm_sys::lib
- Additional types and options for put API calls

### Fixed

- MSRV 1.87
- Removed conflict on MQCHAR and MQBYTE on some platforms

### Other

- All MQI calls ([#50](https://github.com/advantic-au/mqi/pull/50))
- mqsubrq API ([#47](https://github.com/advantic-au/mqi/pull/47))
- Latest rust edition ([#45](https://github.com/advantic-au/mqi/pull/45))
- Support for get bag functions ([#30](https://github.com/advantic-au/mqi/pull/30))
- All MQAI verbs ([#23](https://github.com/advantic-au/mqi/pull/23))
- Test helpers.
- on_unimplemented messages
- dlopen2 support
- CCSID new type
- General API and code cleanups
- forward example - forwards a message
- Refreshed syncpoint code
- Improved documentation
