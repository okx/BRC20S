`ord`
=====

`ord` forks on [casey's](https://github.com/casey/ord) and adds the [BRC20 Protocol](https://domo-2.gitbook.io/brc-20-experiment/) feature. It can easily call the API, obtaining the BRC20 transaction and tick balance.

Installation
------------

`ord` is written in Rust and can be built from
[source](https://github.com/okx/ord).

Once `ord` is installed, you should be able to run `ord --version` on the
command line.

Building
--------

On Debian and Ubuntu, `ord` requires `libssl-dev` when building from source:

```
sudo apt-get install libssl-dev
```

You'll also need Rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To build `ord` from source:

```
git clone https://github.com/okx/ord.git
cd ord
cargo build --release --feature=rollback
```

Once built, the `ord` binary can be found at `./target/release/ord`.

`ord` requires `rustc` version 1.67.0 or later. Run `rustc --version` to ensure you have this version. Run `rustup update` to get the latest stable release.

**notice**: casey `ord` does not deal with block reorganization. The database becomes corrupt when a reorganization occurs.

To enable automatic block reorganization, we introduced Redb's savepoint feature, a database backup in the memory. Bitcoin barely reorganizes after six confirmation blocks, and it is possible to make one savepoint every three blocks and keep up to four savepoints so that data can be backed up at least ten heights back. You can add `--feature=rollback` compilation options to activate this feature.

Contributing
------------

If you wish to contribute there are a couple things that are helpful to know. We
put a lot of emphasis on proper testing in the code base, with three broad
categories of tests: unit, integration and fuzz. Unit tests can usually be found at
the bottom of a file in a mod block called `tests`. If you add or modify a
function please also add a corresponding test. Integration tests try to test
end-to-end functionality by executing a subcommand of the binary. Those can be
found in the [tests](tests) directory. We don't have a lot of fuzzing but the
basic structure of how we do it can be found in the [fuzz](fuzz) directory.

We strongly recommend installing [just](https://github.com/casey/just) to make
running the tests easier. To run our CI test suite you would do:

```
just ci
```

This corresponds to the commands:

```
cargo fmt -- --check
cargo test --all
cargo test --all -- --ignored
```

Have look at the [justfile](justfile) to see some more helpful recipes
(commands). Here are a couple more good ones:

```
just fmt
just fuzz
just doc
just watch ltest --all
```

We also try to follow a TDD (Test-Driven-Development) approach, which means we
use tests as a way to get visibility into the code. Tests have to run fast for that
reason so that the feedback loop between making a change, running the test and
seeing the result is small. To facilitate that we created a mocked Bitcoin Core
instance in [test-bitcoincore-rpc](./test-bitcoincore-rpc).

Syncing
-------

`ord` requires a synced `bitcoind` node with `-txindex` to build the index of
satoshi locations. `ord` communicates with `bitcoind` via RPC.

If `bitcoind` is run locally by the same user, without additional
configuration, `ord` should find it automatically by reading the `.cookie` file
from `bitcoind`'s datadir, and connecting using the default RPC port.

If `bitcoind` is not on mainnet, is not run by the same user, has a non-default
datadir, or a non-default port, you'll need to pass additional flags to `ord`.
See `ord --help` for details.

`bitcoind` RPC Authentication
-----------------------------

`ord` makes RPC calls to `bitcoind`, which usually require a username and
password.

By default, `ord` looks a username and password in the cookie file created by
`bitcoind`.

The cookie file path can be configured using `--cookie-file`:

```
ord --cookie-file /path/to/cookie/file server
```

Alternatively, `ord` can be supplied with a username and password on the
command line:

```
ord --bitcoin-rpc-user foo --bitcoin-rpc-pass bar server
```

Using environment variables:

```
export ORD_BITCOIN_RPC_USER=foo
export ORD_BITCOIN_RPC_PASS=bar
ord server
```

Or in the config file:

```yaml
bitcoin_rpc_user: foo
bitcoin_rpc_pass: bar
```

Logging
--------

`ord` uses [log4rs](https://docs.rs/log4rs/latest/log4rs/) instead of [env_logger](https://docs.rs/env_logger/latest/env_logger/). Set the
`--log-level` argument variable in order to turn on logging. For example, run
the server and show `info`-level log messages and above:

```
$ cargo run server --log-level info
```

New Releases
------------

Release commit messages use the following template:

```
Release x.y.z

- Bump version: x.y.z → x.y.z
- Update changelog
- Update dependencies
- Update database schema version
```
