## sftpman

`sftpman` is a command line application for Linux systems that makes it easy to setup and mount [sshfs](https://github.com/libfuse/sshfs)/[SFTP](https://en.wikipedia.org/wiki/SSH_File_Transfer_Protocol) filesystems.

Configuration data is stored as JSON files in `$XDG_CONFIG_HOME/sftpman` (see the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)) or `$HOME/sftpman`.

Under the hood, [sshfs](https://github.com/libfuse/sshfs) is used to do all the filesystem mounting work.
Filesystems are mounted locally under the `/mnt/sshfs/` directory by default (e.g. `/mnt/sshfs/my-fs`), but custom mount endpoints are also supported.

There's also a companion [🖥️ GUI application](#-gui-application) - [sftpman-iced](https://github.com/spantaleev/sftpman-iced-rs).

💡 **Note**: sftpman used to be written in [Python](https://www.python.org/) (see [sftpman-python](https://github.com/spantaleev/sftpman-python)) and had an [sftman-gtk](https://github.com/spantaleev/sftpman-gtk) GUI frontend (also written in Python). Since this [Rust](https://www.rust-lang.org/) rewrite appeared, the old Python-based applications are no longer maintained.


## 🚀 Installing

### Dependencies

- [sshfs](https://github.com/libfuse/sshfs)

### Installing on ArchLinux

For [ArchLinux](http://www.archlinux.org/), there's an [sftpman](https://aur.archlinux.org/packages/sftpman) package in the [AUR](https://wiki.archlinux.org/title/Arch_User_Repository).

### Installing on other distributions

For other Linux distributions you can install using [cargo](https://doc.rust-lang.org/cargo/):

```sh
cargo install sftpman
```

You also need to:

- install [sshfs](https://github.com/libfuse/sshfs) yourself

- make sure the `/mnt/sshfs` directory exists and is writable (e.g. `mkdir /mnt/sshfs && chown root:users /mnt/sshfs && chmod 0775 /mnt/sshfs`)


## ⌨ CLI Application

Once you've [🚀 installed](#-installing) the CLI application, you can:

- check if your system satisfies the prerequisites by running: `sftpman preflight_check`
- manage filesystems via commands like: `sftpman create`, `sftpman update` and `sftpman rm`, etc.
- mount filesystems via commands like: `sftpman mount my-fs-1 my-fs-2` or `sftpman mount_all`
- unmount filesystems via commands like: `sftpman umount my-fs my-fs-2` or `sftpman umount_all`
- list filesystems via commands like: `sftpman ls available`, `sftman ls mounted` or `sftpman ls unmounted`

See `sftpman --help` for more information.


## 🖥️ GUI Application

[sftpman-iced](https://github.com/spantaleev/sftpman-iced-rs) is a frontend for sftpman built with the [iced](https://iced.rs/) UI library.

💡 **Note**: Installing the GUI application will automatically pull the library provided here (`libsftpman`) as a dependency, but will **not** automatically install the `sftpman` CLI binary.


## ❓ FAQ

### Why not just use sshfs directly?

You certainly can use [sshfs](https://github.com/libfuse/sshfs) directly or wrap it in your own scripts.

`sftpman` aims to be a higher-level wrapper around `sshfs`, so you don't have to do such manual work.

Having a [🖥️ GUI application](#-gui-application) also makes things easier.

### Does sftpman support other protocols (FTP, etc)?

This has been discussed in [this old issue](https://github.com/spantaleev/sftpman-python/issues/8).

As should be evident from the project name (`sftpman`), only [SFTP](https://en.wikipedia.org/wiki/SSH_File_Transfer_Protocol) is being targeted.

Other protocols are out of scope for this project.

### Why was sftpman rewritten from Python to Rust?

The [Python](https://www.python.org/) version of sftpman has worked well for more than a decade (2011-2025), but:

- packaging Python application is somewhat annoying. Major Python version upgrades change the `site-packages` path and necessitate a rebuild.

- [Rust](https://www.rust-lang.org/) is very suitable for systems programs (like `sftpman`). The code is much cleaner and more reliable.

#### Is sftpman v2 compatible with sftpman v1?

`sftpman` v2 (and the [🖥️ GUI application](#-gui-application) - [sftpman-iced](https://github.com/spantaleev/sftpman-iced-rs)) are still **mostly-backward compatible** with the old Python-based `sftpman` software ([sftpman-python](https://github.com/spantaleev/sftpman-python) and the associated [sftman-gtk](https://github.com/spantaleev/sftpman-gtk)):

- ✅ The old configuration files can be read and used as-is

- ✅ Most CLI commands are the same

- ✨ You can now use custom local mount endpoints for filesystems, instead of just the default `/mnt/sshfs/{id}` directory

- ❌ Some CLI commands for `sftpman` and their options have different names (`sftpman setup` being replaced by `sftpman create` and `sftpman update`; `--auth_method` being replaced by `--auth_type`)
