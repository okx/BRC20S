[ARCHIVED]`ord`
=====
> ### This repo has been archived
> With the BRC20 `Jubilee` upgrade, we have decided to suspend maintenance of this repo. Details as follows:
> 1. Cease maintenance of the BRC20S protocol implementation.
> 2. The BRC20 protocol implementation that supports the 'Jubilee' upgrade has moved to [okx/ord](https://github.com/okx/ord).

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
cargo build --release
```

Once built, the `ord` binary can be found at `./target/release/ord`.

`ord` requires `rustc` version 1.67.0 or later. Run `rustc --version` to ensure you have this version. Run `rustup update` to get the latest stable release.

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

Have a look at the [justfile](justfile) to see some more helpful recipes
(commands). Here are a couple more good ones:

```
just fmt
just fuzz
just doc
just watch ltest --all
```

If the tests are failing or hanging, you might need to increase the maximum
number of open files by running `ulimit -n 1024` in your shell before you run
the tests, or in your shell configuration.

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

`ord` makes RPC calls to `bitcoind`, which usually requires a username and
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

SnapShot
--------
Use a snapshot to quickly synchronize the BRC20S indexer database.

1. Download the specified height snapshot database from this web page.
- <https://static.okex.org/cdn/chain/brc20s/snapshot/history-brc20s.html>

2. Extract and Unzip the `.tar.gz` file and replace the database file.

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

Translations
------------

To translate [the docs](https://docs.ordinals.com) we use this
[mdBook i18n helper](https://github.com/google/mdbook-i18n-helpers).
So read through their [usage guide](https://github.com/google/mdbook-i18n-helpers/blob/main/USAGE.md)
to see the structure that translations should follow.

There are some other things to watch out for but feel free to just start a
translation and open a PR. Have a look at [this commit](https://github.com/ordinals/ord/commit/329f31bf6dac207dad001507dd6f18c87fdef355)
for an idea of what to do. A maintainer will also help you integrate it into our
build system.

To align your translated version of the Handbook with reference to commit
[#2427](https://github.com/ordinals/ord/pull/2426), here are some guiding
commands to assist you. It is assumed that your local environment is already
well-configured with [Python](https://www.python.org/),
[Mdbook](https://github.com/rust-lang/mdBook),
[mdBook i18n helper](https://github.com/rust-lang/mdbb) and that you've clone
this repo.


1. Run the following command to generate a new `pot` file, which is named as
`messages.pot`:

```
MDBOOK_OUTPUT='{"xgettext": {"pot-file": "messages.pot"}}'
mdbook build -d po
```

2. Run `msgmerge` where `xx.po` is your localized language version following
the naming standard of [ISO639-1](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes).
This process will update the `po` file with the most recent original version:

```
msgmerge --update po/xx.po po/messages.pot
```

3. Look for `#, fuzzy`. The `mdBook-i18n-helper` tool utilizes the `"fuzzy"` tag
to highlight sections that have been recently edited. You can proceed to perform
the translation tasks by editing the `"fuzzy"`part.

4. Execute the `mdbook` command. A demonstration in Chinese (`zh`) is given below:

```
mdbook build docs -d build
MDBOOK_BOOK__LANGUAGE=zh mdbook build docs -d build/zh
mv docs/build/zh/html docs/build/html/zh
python3 -m http.server --directory docs/build/html --bind 127.0.0.1 8080
```

5. Upon verifying everything and ensuring all is in order, you can commit the
modifications and progress to open a Pull Request (PR) on Github.
(**Note**: Please ensure **ONLY** the **'xx.po'** file is pushed, other files
such as '.pot' or files ending in '~' are **unnecessary** and should **NOT** be
included in the Pull Request.）
