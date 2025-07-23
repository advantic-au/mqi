# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/advantic-au/mqi/compare/v0.2.0...v0.3.0) - 2025-07-23

### Added

- ConnectionEither to hold either ConnectionRef or Connection

### Fixed

- ci warnings
- memory leak removed
- release-plz permission
- ConnectionRef leak of Library

### Other

- MQAI trait
- latest dependencies
- latest dependencies
- *(fix)* feature gate mock on tests
- Further tightening of the API definition
- Event Handler mocking
- Syncpoint and Callback simplify
- Replaced Conn with AsConnection
- Safe event handler callback implementation
- Version assertions
- Remove mqm_generate
- doctest fixes
- Rework ConnectionRef and simplify Conn
- further refinement of api into modules
- Moved all handles into a handle module.
- handles rework
- Replaced MQMD2 with MQMD
- mqai src structure refactor
- QueueManager remove and callback move
- refactor stat file structure
- move attribute and constants use locations
- reorganise code - compile working
- Major semver
- docsrs cleanup

## [0.2.0](https://github.com/advantic-au/mqi/compare/v0.1.0...v0.2.0) - 2025-07-07

### Added

- Added "mqc_latest" feature for latest MQ version dependency
- Removed re pub of libmqm_sys::lib
- Additional types and options for put API calls

### Fixed

- clippy warnings with beta
- Address CI errors after libmqm_sys upgrade
- add feature gate for InitialKey
- Move InitialKey into its own ConnectionOption
- compile of object tests
- type "need"
- Property options for MqStr
- MSRV 1.87
- Removed conflict on MQCHAR and MQBYTE on some platforms
- docs-rs check for mqconnx
- document reference fixes
- conditional mockfunctions for mqai
- clippy matrix

### Other

- Remove github for libmqm-* dependency ([#82](https://github.com/advantic-au/mqi/pull/82))
- Bump libmqm-* dependencies
- bump install-action
- rustfmt
- Merge remote-tracking branch 'advantic/dependabot/github_actions/taiki-e/install-action-2.52.4' into release_prep
- Update to latest libmqm_sys version 0.10
- restructured imports
- Internal document link fixes
- Flatten source code structure
- restructure preparation
- Merge remote-tracking branch 'advantic/dependabot/github_actions/taiki-e/install-action-2.52.2' into release_prep
- Merge remote-tracking branch 'advantic/dependabot/github_actions/MarcoIeni/release-plz-action-0.5.107' into release_prep
- Merge remote-tracking branch 'advantic/dependabot/github_actions/astral-sh/setup-uv-6.1.0' into release_prep
- Merge remote-tracking branch 'advantic/dependabot/github_actions/dtolnay/rust-toolchain-b3b07ba8b418998c39fb20f53e8b695cdcc8de1b' into release_prep
- Remove regex-lite depenendency
- fix minimum version build for tracing-subscriber
- test on minimum job
- doctest with no default features fixed
- Removed redundant page_size dependency
- fix old Credentials::user function call
- ensure mocking is used in test
- Fix broken links to libmqm_sys
- test various feature combinations
- docsrs feature dependency on document-features
- added outcome jobs
- clippy dependency on MQ client. tests us matrix.os
- add test on arm64
- removed runnable feature
- Test ignore reason
- Use "raw" references.
- Added testing for name retrieval for Properties::property
- Coherent and thorough unit testing of Properties::property
- Fixed some safety descriptions
- * Added TLS capability to examples
- ConnectStructFlags, simplified lifetimes
- Additional PropertyValue unit tests
- Additional PropertyValue test cases
- Additional PropertyValue unit tests
- rustfmt and nightly clippy fixes
- depend on libmqm-* git repo in development
- Further use of constants
- Re
- Use uv to install zizmor
- Explicit stable / beta configuration
- Quote clippy parameters
- Fix zizmor audit findings
- ci fixes
- libmqm-sys major version bump
- MQ 9.4.2 pregen ([#54](https://github.com/advantic-au/mqi/pull/54))
- MQ client 9.4.2 ([#53](https://github.com/advantic-au/mqi/pull/53))
- Remaining MQI calls ([#50](https://github.com/advantic-au/mqi/pull/50))
- *(deps)* bump taiki-e/install-action from 2.48.20 to 2.49.0 ([#46](https://github.com/advantic-au/mqi/pull/46))
- mqsubrq API ([#47](https://github.com/advantic-au/mqi/pull/47))
- Latest rust edition ([#45](https://github.com/advantic-au/mqi/pull/45))
- Support for get bag functions ([#30](https://github.com/advantic-au/mqi/pull/30))
- *(deps)* bump taiki-e/install-action from 2.48.13 to 2.48.20 ([#44](https://github.com/advantic-au/mqi/pull/44))
- version upkeep ([#43](https://github.com/advantic-au/mqi/pull/43))
- static analysis on push to develop ([#38](https://github.com/advantic-au/mqi/pull/38))
- ci/cd vulnerability remediation ([#37](https://github.com/advantic-au/mqi/pull/37))
- Remaining MQI function wrapping ([#35](https://github.com/advantic-au/mqi/pull/35))
- Clippy run on stable and beta toolchain ([#33](https://github.com/advantic-au/mqi/pull/33))
- Documentation improvements ([#31](https://github.com/advantic-au/mqi/pull/31))
- CI tweaks ([#29](https://github.com/advantic-au/mqi/pull/29))
- Parameter references ([#28](https://github.com/advantic-au/mqi/pull/28))
- miri fixes ([#26](https://github.com/advantic-au/mqi/pull/26))
- Refinement of string references ([#27](https://github.com/advantic-au/mqi/pull/27))
- *(ci)* dependabot conventional commits ([#25](https://github.com/advantic-au/mqi/pull/25))
- Remaining MQAI verbs ([#23](https://github.com/advantic-au/mqi/pull/23))
- Test coverage features ([#24](https://github.com/advantic-au/mqi/pull/24))
- fix mutants testing ([#22](https://github.com/advantic-au/mqi/pull/22))
- Incremental mutant testing ([#21](https://github.com/advantic-au/mqi/pull/21))
- Allow release-plz to work ([#20](https://github.com/advantic-au/mqi/pull/20))
- Test Coverage ([#18](https://github.com/advantic-au/mqi/pull/18))
- CI nextest ([#17](https://github.com/advantic-au/mqi/pull/17))
- miri github action ([#16](https://github.com/advantic-au/mqi/pull/16))
- next libmqmsys ([#15](https://github.com/advantic-au/mqi/pull/15))
- github action: disable mq web server ([#14](https://github.com/advantic-au/mqi/pull/14))
- github action: allow dispatch ([#13](https://github.com/advantic-au/mqi/pull/13))
- Server testing ([#12](https://github.com/advantic-au/mqi/pull/12))
- Nightly Github Actions ([#11](https://github.com/advantic-au/mqi/pull/11))
- PutOption refactor ([#10](https://github.com/advantic-au/mqi/pull/10))
- Mocking ([#9](https://github.com/advantic-au/mqi/pull/9))
- Parameter simplification ([#8](https://github.com/advantic-au/mqi/pull/8))
- github actions: add MQ client download for release-plz-release job
- github actions: release-plz ([#5](https://github.com/advantic-au/mqi/pull/5))
- regenerate pregen constants ([#6](https://github.com/advantic-au/mqi/pull/6))
- pregen and docsrs features depend on latest MQ client
- MQ version feature flags ([#4](https://github.com/advantic-au/mqi/pull/4))
- Github Actions: MQ client 9.4.1.0
- Dependabot
- Merge branch 'develop' of github.com:advantic-au/mqi into develop
- GitHub Action: Add CI for MQ client 9.2 - current
- github actions: pregen command
- github action: pregen check
- rustfmt
- docsrs unused_variables
- Remove warnings on docsrs build
- clippy action and docsrs warning fix
- github action: first attempt
- pregen feature
- Create identifier type
- Added test helpers.
- Additional on_unimplemented messages
- Removed MqiOption.
- dlopen2 support and where clauses
- Variable name consistency
- CCSID new type
- Reinstate CB
- Removed QueueManager wrapper
- Shift execute to QueueManager.
- QueueManager type rejig
- Identifier naming cleanup.
- Added try run forward example
- forward example - forwards a message
- Refreshed syncpoint code
- mqmask and mqvalue rustdoc
- Improved documentation.
