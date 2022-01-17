# scouty &middot; ![latest release](https://github.com/turboflakes/scouty/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/scouty/blob/main/assets/scouty-github-header.png?raw=true">
</p>

`scouty` is a command-line interface (CLI) to keep an **eye** on substrate-based chains and **hook** things up.

## Why use `scouty`

To get **notified** about on-chain events emitted by certain operations linked to *Session*, *Staking*, *ImOnline* and *Democracy* pallets.

To monitor, intercept and **extend functionality** as soon as on-chain events are triggered.

To get access to on-chain data and customize messages **written by you** delivered to a matrix **private** room.

To warm up or cool down your validator node resources by knowing when it goes active or inactive within one session.

To **monitor 1KV nominations** and trigger special kudos when your validator becomes independent :)

To keep up with **Referenda** and vote from your favorite site - *Polkadot.js, Polkassembly, Commonwealth* - through a direct link.

To trigger **node backups** every other era and publish them online.

To write **your own bash scripts** and hook them up to any on-chain event supported by `scouty`.

## Hooks 🪝

`scouty v0.1.18` supports 9 native hooks ready to be explored:

- Everytime `scouty` **starts** the following hook is executed ->  [`_init.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_init.sh) (Note: This hook can be used to try out and test new scripts)
- At every **New Era** the following hook is executed ->  [`_new_era.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_new_era.sh)
- At every **New Session** the following hook is executed ->  [`_new_session.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_new_session.sh)
- Everytime a **Referendum Starts** the following hook is executed ->  [`_democracy_started.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_democracy_started.sh)
- At the begining of the last session of an era, if a validator is in the **waiting set** and is **queued** to be **active in the next era**, the following hook is executed ->  [`_validator_starts_active_next_era.s`](https://github.com/turboflakes/scouty/tree/main/hooks/_validator_starts_active_next_era.sh) (Note: only executed for the stashes predefined)
- At the begining of the last session of an era, if a validator is in the **active set** and is **NOT queued** to be active in the next era, the following hook is executed ->  [`_validator_starts_inactive_next_era.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_validator_starts_inactive_next_era.sh) (Note: only executed for the stashes predefined)
- Everytime a validator is **Chilled** the following hook is executed ->  [`_validator_chilled.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_validator_chilled.sh) (Note: only executed for the stashes predefined)
- Everytime a **Slash occurred** the following hook is executed ->  [`_validator_slashed.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_validator_slashed.sh)
- At the end of every era, if a **validator is seen to be Offline** the following hook is executed ->  [`_validator_offline.sh`](https://github.com/turboflakes/scouty/tree/main/hooks/_validator_offline.sh) (Note: only executed for the stashes predefined)

### The possibilities are endless ✨

A few example scripts are available here -> [hooks.examples](https://github.com/turboflakes/scouty/tree/main/hooks.examples). I encourage you to try out your *bash* scripts with `scouty` and please feedback and share some examples with the community by submitting a pull request [here](https://github.com/turboflakes/scouty/tree/main/hooks.examples).

Note: By default every hook is followed by a custom Matrix message. Read [here](https://github.com/turboflakes/scouty#scouty-bot-matrix) on how to setup -> Scouty Bot.

## Installation

```bash
#!/bin/bash
# create `scouty-cli` directory
mkdir /opt/scouty-cli
# download `scouty` binary latest version
wget -P /scouty-cli https://github.com/turboflakes/scouty/releases/download/v0.1.18/scouty
# make `scouty` binary file executable
chmod +x /opt/scouty-cli/scouty
```

Note: For an easier installation and faster updates download [`scouty-update.sh`](https://github.com/turboflakes/scouty/blob/main/scouty-update.sh) bash script file and make it executable.

## Configuration

First create a configuration file `.env` inside `scouty-cli` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/scouty/blob/main/.env.example) (Note: `.env` is the default name and a hidden file, if you want something different you can adjust it later with the option `scouty --config-path /opt/scouty-cli/.env.kusama` )

```bash
touch /opt/scouty-cli/.env
```

Open the file (using Vim in this case) and add/change the configuration variables with your own values.

```bash
vi /opt/scouty-cli/.env
```

Configuration file example: [`.env.example`](https://github.com/turboflakes/scouty/blob/main/.env.example)

```bash
# scouty CLI configuration variables 
#
# [SCOUTY_STASHES] Validator stash addresses for which 'scouty' will be applied. 
# If needed specify more than one (e.g. stash_1,stash_2,stash_3).
SCOUTY_STASHES=5GTD7ZeD823BjpmZBCSzBQp7cvHR1Gunq7oDkurZr9zUev2n
#
# [SCOUTY_SUBSTRATE_WS_URL] Substrate websocket endpoint for which 'scouty' will try to
# connect. (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence
# than <CHAIN> argument) 
SCOUTY_SUBSTRATE_WS_URL=wss://localhost:9944
#SCOUTY_SUBSTRATE_WS_URL=wss://westend-rpc.polkadot.io:443
#
# Hooks configuration bash script filenames
SCOUTY_HOOK_INIT_PATH=/opt/scouty-cli/hooks/_init.sh
SCOUTY_HOOK_NEW_SESSION_PATH=/opt/scouty-cli/hooks/_new_session.sh
SCOUTY_HOOK_NEW_ERA_PATH=/opt/scouty-cli/hooks/_new_era.sh
SCOUTY_HOOK_VALIDATOR_STARTS_ACTIVE_NEXT_ERA_PATH=/opt/scouty-cli/hooks/_validator_starts_active_next_era.sh
SCOUTY_HOOK_VALIDATOR_STARTS_INACTIVE_NEXT_ERA_PATH=/opt/scouty-cli/hooks/_validator_starts_inactive_next_era.sh
SCOUTY_HOOK_VALIDATOR_SLASHED_PATH=/opt/scouty-cli/hooks/_validator_slashed.sh
SCOUTY_HOOK_VALIDATOR_CHILLED_PATH=/opt/scouty-cli/hooks/_validator_chilled.sh
SCOUTY_HOOK_VALIDATOR_OFFLINE_PATH=/opt/scouty-cli/hooks/_validator_offline.sh
SCOUTY_HOOK_DEMOCRACY_STARTED_PATH=/opt/scouty-cli/hooks/_democracy_started.sh
#
# Matrix configuration variables
SCOUTY_MATRIX_USER=@your-regular-matrix-account:matrix.org
SCOUTY_MATRIX_BOT_USER=@your-own-scouty-bot-account:matrix.org
SCOUTY_MATRIX_BOT_PASSWORD=anotthateasypassword
#
# when ready write and quit (:wq!)
```

### Run `scouty` as a *systemd* service

First create a unit file called `scouty.service` in `/etc/systemd/system/`

```bash
touch /etc/systemd/system/scouty.service
```

Below there is an example of a service configuration file for reference

```bash
[Unit]
Description=Scouty Bot

[Service]
ExecStart=/opt/scouty-cli/scouty --config-path /opt/scouty-cli/.env
Restart=always
RestartSec=15

[Install]
WantedBy=multi-user.target
```

Enable, start and check the status of the service

```bash
systemctl enable scouty.service
systemctl start scouty.service
systemctl status scouty.service
```

To look out for tailed logs with `journalctl` run

```bash
journalctl -f -u scouty
```

If you have started scouty by now you should get these warnings in your logs
```bash
WARN  scouty::hooks] Hook script - Scouty initialized - filename (/opt/scouty-cli/hooks/_init.sh) not defined
WARN  scouty::hooks] Hook script - New session - filename (/opt/scouty-cli/hooks/_new_session.sh) not defined
WARN  scouty::hooks] Hook script - New era - filename (/opt/scouty-cli/hooks/_new_era.sh) not defined
WARN  scouty::hooks] Hook script - Validator starts active next era - filename (/opt/scouty-cli/hooks/_validator_starts_active_next_era.sh) not defined
WARN  scouty::hooks] Hook script - Validator starts inactive next era - filename (/opt/scouty-cli/hooks/_validator_starts_inactive_next_era.sh) not defined
WARN  scouty::hooks] Hook script - Validator has been slashed - filename (/opt/scouty-cli/hooks/_validator_slashed.sh) not defined
WARN  scouty::hooks] Hook script - Validator has been chilled - filename (/opt/scouty-cli/hooks/_validator_chilled.sh) not defined
WARN  scouty::hooks] Hook script - Validator has been offline - filename (/opt/scouty-cli/hooks/_validator_offline.sh) not defined
WARN  scouty::hooks] Hook script - Democracy started - filename (/opt/scouty-cli/hooks/_democracy_started.sh) not defined
```

These are just warnings to tell you that those `bash script` files are not available and `scouty` will not be able to run them.

So let's just set these up as our last step. Create a sub directory `hooks` inside `scouty-cli`

```bash
 mkdir /opt/scouty-cli/hooks
```

If you have cloned the repo on your local machine you can securely copy the default hook files from [here](https://github.com/turboflakes/scouty/tree/main/hooks) or copy some of the [examples](https://github.com/turboflakes/scouty/tree/main/hooks.examples) to your remote server with `scp`

```bash
scp -r ./hooks/* root@IPADDRESS:/opt/scouty-cli/hooks
```

And make these hooks executable by running

```bash
chmod +x /opt/scouty-cli/hooks/_init.sh
chmod +x /opt/scouty-cli/hooks/_new_session.sh
chmod +x /opt/scouty-cli/hooks/_new_era.sh
chmod +x /opt/scouty-cli/hooks/_validator_starts_active_next_era.sh
chmod +x /opt/scouty-cli/hooks/_validator_starts_inactive_next_era.sh
chmod +x /opt/scouty-cli/hooks/_validator_slashed.sh
chmod +x /opt/scouty-cli/hooks/_validator_chilled.sh
chmod +x /opt/scouty-cli/hooks/_democracy_started.sh
```

Finally restart `scouty` *systemd* service

```bash
systemctl restart scouty.service
```

### Scouty Bot ([Matrix](https://matrix.org/))

If you set up `scouty` on your server with a matrix user 👉 you get your own Scouty Bot.

To enable **Scouty Bot** you will need to create a specific account on Element or similar and copy the values to the respective environment variables `SCOUTY_MATRIX_BOT_USER` and `SCOUTY_MATRIX_BOT_PASSWORD` like in the configuration example file `.env.example`. You may also want to set your regular matrix user to the environment variable `SCOUTY_MATRIX_USER`. So that **Scouty Bot** could create a private room and send in messages. By default **Scouty Bot** will automatically invite your regular matrix user to a private room.

### Scouty Bot hook message [examples](https://github.com/turboflakes/scouty/tree/main/assets)

#### _democracy_started

<p align="left">
    <img  style="width: 384px;" src="https://github.com/turboflakes/scouty/blob/main/assets/matrix_democracy_started.png?raw=true">
</p>

#### _validator_starts_active_next_era

<p align="left">
    <img  style="width: 384px;" src="https://github.com/turboflakes/scouty/blob/main/assets/matrix_validator_starts_active_next_era.png?raw=true">
</p>

#### _validator_slashed

<p align="left">
    <img  style="width: 384px;" src="https://github.com/turboflakes/scouty/blob/main/assets/matrix_validator_slashed.png?raw=true">
</p>

## Usage

Run `--help` to check all `scouty` flags and options.

Note: All flags and options are also available through environment variables if defined in `.env` configuration file. You can choose which way you want to configure `scouty`. Take in consideration that if the same variable is defined on both sides e.g. defined in `.env` and through CLI flag/option, `scouty` will take the value defined by CLI.

```bash
#!/bin/bash
# if you need a custom scouty check all the options and flags available
scouty --help
```

```bash
USAGE:
    scouty [FLAGS] [OPTIONS] [CHAIN]

FLAGS:
        --debug                              Prints debug information verbosely.
        --disable-matrix                     Disable matrix bot for 'scouty'. (e.g. with this flag active 'scouty' will
                                             not send messages/notifications to your private 'Scouty Bot' room)
                                             (https://matrix.org/)
        --disable-matrix-bot-display-name    Disable matrix bot display name update for 'scouty'. (e.g. with this flag
                                             active 'scouty' will not change the matrix bot user display name)
        --expose-network                     Expose the network name, token symbol and token decimal under new
                                             positional arguments for each hook.
        --expose-nominators                  Expose the nominator details under new positional arguments for some of the
                                             hooks. Note: `scouty` only look after active nominators for each validator
                                             stash predefined.
    -h, --help                               Prints help information
        --short                              Display only essential information (e.g. with this flag active 'scouty'
                                             will hide certain sections in a message)
    -V, --version                            Prints version information

OPTIONS:
    -c, --config-path <FILE>
            Sets a custom config file path. The config file contains 'scouty' configuration variables. [default: .env]

        --error-interval <error-interval>
            Interval value (in minutes) from which 'scouty' will restart again in case of a critical error. [default:
            30]
        --hook-init-path <FILE>
            Sets the path for the script that is called every time `scouty` starts. Here is a good place for try out new
            things and test new scripts.
        --hook-new-era-path <FILE>
            Sets the path for the script that is called every new era.

        --hook-new-session-path <FILE>
            Sets the path for the script that is called every new session.

        --hook-validator-chilled-path <FILE>
            Sets the path for the script that is called every time one of the Validator stashes defined is chilled.

        --hook-validator-offline-path <FILE>
            Sets the path for the script that is called every time one of the Validator stashes defined is offline at
            the end of a session.
        --hook-validator-slashed-path <FILE>
            Sets the path for the script that is called every time a Slash occurred on the network.

        --hook-validator-starts-active-next-era-path <FILE>
            Sets the path for the script that is called on the last session of an era, if the stash is NOT ACTIVE and
            keys are QUEUED for the next Session/Era.
        --hook-validator-starts-inactive-next-era-path <FILE>
            Sets the path for the script that is called on the last session of an era, if the stash is ACTIVE and keys
            are NOT QUEUED for the next Session/Era.
        --matrix-bot-password <matrix-bot-password>              Password for the 'Scouty Bot' matrix user sign in.
        --matrix-bot-user <matrix-bot-user>
            Your new 'Scouty Bot' matrix user. e.g. '@your-own-scouty-bot-account:matrix.org' this user account will be
            your 'Scouty Bot' which will be responsible to send messages/notifications to your private 'Scouty Bot'
            room.
        --matrix-user <matrix-user>
            Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive
            notifications from your other 'Scouty Bot' matrix account.
    -s, --stashes <stashes>
            Validator stash addresses for which 'scouty' will take a particular eye. If needed specify more than one
            (e.g. stash_1,stash_2,stash_3).
    -w, --substrate-ws-url <substrate-ws-url>
            Substrate websocket endpoint for which 'scouty' will try to connect. (e.g. wss://kusama-rpc.polkadot.io)
            (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)

ARGS:
    <CHAIN>    Sets the substrate-based chain for which 'scouty' will try to connect [possible values: westend,
               kusama, polkadot]
```

## Development / Build from Source

If you'd like to build from source, first install Rust.

```bash
curl https://sh.rustup.rs -sSf | sh
```

If Rust is already installed run

```bash
rustup update
```

Verify Rust installation by running

```bash
rustc --version
```

Once done, finish installing the support software

```bash
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Build `scouty` by cloning this repository

```bash
#!/bin/bash
git clone http://github.com/turboflakes/scouty
```

Compile `scouty` package with Cargo

```bash
#!/bin/bash
cargo build
```

And then run it

```bash
#!/bin/bash
./target/debug/scouty
```

Otherwise, recompile the code on changes and run the binary

```bash
#!/bin/bash
cargo watch -x 'run --bin scouty'
```

### Downloading metadata from a Substrate node

Use the [`subxt-cli`](./cli) tool to download the metadata for your target runtime from a node.

Install
```bash
cargo install subxt-cli
```
Save the encoded metadata to a file
```bash
subxt metadata --url https://westend-rpc.polkadot.io  -f bytes > westend_metadata.scale
```
(Optional) Generate runtime API client code from metadata
```bash
subxt codegen --url https://westend-rpc.polkadot.io | rustfmt --edition=2018 --emit=stdout > westend_runtime.rs
```

## Collaboration

Have an idea for a new feature, a fix or you found a bug, please open an [issue](https://github.com/turboflakes/scouty/issues) or submit a [pull request](https://github.com/turboflakes/scouty/pulls).

Any feedback is welcome.

## About

`scouty` was made by <a href="https://turboflakes.io" target="_blank">TurboFlakes</a>.

If you like this project 💯  
  - 🚀 Share our work 
  - ✌️ Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a>
  - ✨ Or you could also star the Github project :)

### License

`scouty` is [MIT licensed](./LICENSE).

### Quote

> "Everything you can imagine is real." -- Pablo Picasso

__

Enjoy `scouty` and hook things up
