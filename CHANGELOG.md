# 2025-01-07 (Version 2.0.1)

- (**Bugfix**) Make persisting ensure config directory path (`$XDG_CONFIG_HOME/sftpman/mounts`) is created. Fixes [issue #1](https://github.com/spantaleev/sftpman-rs/issues/1).

# 2025-01-07 (Version 2.0.0)

Initial release of the [Rust](https://www.rust-lang.org/) version of sftpman, superseding the old software written in [Python](https://www.python.org/) (see [sftpman-python](https://github.com/spantaleev/sftpman-python) and the associated [sftman-gtk](https://github.com/spantaleev/sftpman-gtk) GUI frontend).

The library included here (`libsftpman`) also powers the new [sftpman-iced](https://github.com/spantaleev/sftpman-iced-rs) GUI frontend.

If upgrading from the v1 versions of sftpman, see [Is sftpman v2 compatible with sftpman v1?](README.md#is-sftpman-v2-compatible-with-sftpman-v1) for more details.
