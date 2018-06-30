# snail [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates]

[travis-img]:   https://travis-ci.org/kpcyrd/snail.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/snail
[crates-img]:   https://img.shields.io/crates/v/snail.svg
[crates]:       https://crates.io/crates/snail

**Disclaimer:** The project is in a very early state, you're very likely to
experience bugs. I'm using it as my daily driver, but you should expect a very
bare bone experience if you're considering doing the same.

Parasitic network manager. snail is trying to fill the gap of a metasploit-like
network manager. Its core feature is a scripting engine that can be used to
match known networks and also interact with captive portals if one is
discovered. Have a look at the [scripts/](scripts/) folder for examples. The
basic idea is that you're trying to get connectivity, but you don't really care
where it's actually coming from. Please remain seated and keep your arms and
legs inside the firewall at all times.

![logo](docs/logo.png)

## Installation

If possible, use the [snail-git] package for archlinux. For a manual setup on a
debian based system, install the dependency libraries `libseccomp-dev`,
`libdbus-1-dev` and `libzmq3-dev`.

[snail-git]: https://aur.archlinux.org/packages/snail-git/

Next, build the binary:
```
cargo build --release
```

And install it:
```
install -Dm755 target/release/snail{d,ctl} /usr/bin
install -Dm644 scripts/* -t /usr/lib/snaild/scripts

install -d /etc/snail/scripts
install -Dm644 contrib/snail.conf -t /etc/snail
install -Dm644 contrib/snail@.service -t /usr/lib/systemd/system

systemctl daemon-reload
systemctl enable --now snail@wlp3s0
```

You can monitor your network status with snailctl. Make sure your user is in
the correct group which is specified in /etc/snail/snail.conf.
```
snailctl status
```

## Trivia

The name snailctl is inspired by [Leucochloridium], a parasite that lives
inside a snail and is able to control its host. In the long term, `snaild` is
the process that interfaces with the snails brain and `snailctl` is the utility
to control the snail through `snaild`.

The logo has been created by Baudon in 1879.

[Leucochloridium]: https://en.wikipedia.org/wiki/Leucochloridium

## License

GPLv3+
