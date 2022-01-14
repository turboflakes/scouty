# scouty &middot; ![latest release](https://github.com/turboflakes/scouty/actions/workflows/create_release.yml/badge.svg)

`scouty` is a command-line interface (CLI) to keep an **eye** on substrate-based chains and **hook** things up.

## Why use `scouty-cli`

To get **notified** about important on-chain events linked to Session, Staking, ImOnline and Democracy pallets.

To monitor, intercept and **extend functionality** on certain on-chain events.

To get access to on-chain data and customize messages **written by you** delivered to a matrix **private** room.

To warm up or cool down you validator node resources by knowing when it goes active or inactive within one session.

To keep up with **Referenda** and vote on your favorite site - *Polkadot.js, Polkassembly, Commonwealth* - through a custom link.

To write **your own bash scripts** and hook them up to any on-chain event subscribed by `scouty`.

## The possibilities are endless ✨

A few example scripts are available here -> [hooks.examples](https://github.com/turboflakes/scouty/tree/main/hooks.examples). I encourage you to try out your *bash* scripts with `scouty` and please share some examples with the community by pushing a PR [here](https://github.com/turboflakes/scouty/tree/main/hooks.examples).

## Installation

```bash
#!/bin/bash
# create `scouty-cli` directory
mkdir /opt/scouty-cli
# download `scouty` binary latest version
wget -P /scouty-cli https://github.com/turboflakes/scouty/releases/download/v0.1.13/scouty
# make `scouty` binary file executable
chmod +x /opt/scouty-cli/scouty
```

Note: For an easier installation and faster updates download [`scouty-update.sh`](https://github.com/turboflakes/scouty/blob/main/scouty-update.sh) bash script file and make it executable.

## Configuration

Create a configuration file `.env` inside `scouty-cli` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/scouty/blob/main/.env.example) (Note: `.env` is the default name and a hidden file, if you want something different you can adjust it later with the option `scouty --config-path /opt/scouty-cli/.env.kusama` )

```bash
#!/bin/bash
# create/open a file with a file editor (Vim in this case) and add/change the configuration
# variables with your own personal values
vi /scouty-cli/.env
# when ready write and quit (:wq!)
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
```

### Run `scouty` a *systemd* service

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
journalctl -f -u polkadot-validator
```

### Scouty Bot ([Matrix](https://matrix.org/))

If you set up `scouty` on your server with a matrix user 👉 you get your own Scouty Bot.

To enable **Scouty Bot** you will need to create a specific account on Element or similar and copy the values to the respective environment variables `SCOUTY_MATRIX_BOT_USER` and `SCOUTY_MATRIX_BOT_PASSWORD` like in the configuration example file `.env.example`. You may also want to set your regular matrix user to the environment variable `SCOUTY_MATRIX_USER`. So that **Scouty Bot** could create a private room and send in messages. By default **Scouty Bot** will automatically invite your regular matrix user to a private room.

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

Build `crunch` by cloning this repository

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

## Collaboration

Have an idea for a new feature, a fix or you found a bug, please open an [issue](https://github.com/turboflakes/scouty/issues) or submit a [pull request](https://github.com/turboflakes/scouty/pulls).

Any feedback is welcome.

# About

`scouty` was made by <a href="https://turboflakes.io" target="_blank">TurboFlakes</a>.

If you like this project 💯  
  - 🚀 Share our work 
  - ✌️ Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a>
  - ✨ Or you could also star the Github project :)

### License

`scouty` is [MIT licensed](./LICENSE).

### Quote

> "Everything you can imagine is real."
-- Pablo Picasso

__

Enjoy `scouty`
