# 2025-03-07 (Version 2.0.5)

- (**Bugfix**) Wrapping hosts in `[]` when passing them to the `sshfs` command to allow usage with IPv6 addresses

# 2025-02-07 (Version 2.0.4)

- Dependency updates

- (**Bugfix**) Not erroring-out when the mounts configuration directory (e.g. `~/.config/sftpman/mounts`) doesn't exist yet (which is the case for new installations)

# 2025-01-12 (Version 2.0.3)

- (**Bugfix**) Fix mount options not being added when constructing the `sshfs` command ([PR #2](https://github.com/spantaleev/sftpman-rs/pull/2) by [Roald Clark](https://github.com/roaldclark)).

# 2025-01-07 (Version 2.0.2)

- (**Bugfix**) Fix license information discrepancy (GPL v3 -> AGPLv3).

# 2025-01-07 (Version 2.0.1)

- (**Bugfix**) Make persisting ensure config directory path (`$XDG_CONFIG_HOME/sftpman/mounts`) is created. Fixes [issue #1](https://github.com/spantaleev/sftpman-rs/issues/1).

# 2025-01-07 (Version 2.0.0)

Initial release of the [Rust](https://www.rust-lang.org/) version of sftpman, superseding the old software written in [Python](https://www.python.org/) (see [sftpman-python](https://github.com/spantaleev/sftpman-python) and the associated [sftman-gtk](https://github.com/spantaleev/sftpman-gtk) GUI frontend).

The library included here (`libsftpman`) also powers the new [sftpman-iced](https://github.com/spantaleev/sftpman-iced-rs) GUI frontend.

If upgrading from the v1 versions of sftpman, see [Is sftpman v2 compatible with sftpman v1?](README.md#is-sftpman-v2-compatible-with-sftpman-v1) for more details.
