# Fish Folk: Punchy

[![Build Status](https://img.shields.io/github/workflow/status/fishfolks/punchy/CI?logo=github&labelColor=1e1c24&color=216e9b)](https://github.com/fishfolks/punchy/actions)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg?label=license&labelColor=1e1c24&color=34925e)](./LICENSE.md)
[![Discord](https://img.shields.io/badge/chat-on%20discord-green.svg?logo=discord&logoColor=fff&labelColor=1e1c24&color=8d5b3f)](https://discord.gg/4smxjcheE5)
[![Bors enabled](https://bors.tech/images/badge_small.svg)](https://app.bors.tech/repositories/46829)

A 2.5D side-scroller beat-’em-up, made in Bevy. Inspired by games like Little Fighter 2, River City Ransom and [many more](https://fextralife.com/a-history-of-the-side-scrolling-beat-em-up-part-1/).

![EA469655-50B7-487F-84EA-A4A06938356A](https://user-images.githubusercontent.com/583842/161245719-7b587a2a-dd02-4edc-8640-b26ae6f7eafb.gif)

https://user-images.githubusercontent.com/1379590/215623680-2abbd867-717c-42d3-898c-aa7d110e0e0c.mp4

## Web Build

We keep our web build current with our [latest relese](https://github.com/fishfolk/punchy/releases/latest).

https://fishfolk.github.io/punchy/player/latest/

Previous versions of the web build can also be found linked on their [respective release pages](https://github.com/fishfolk/punchy/releases/).

## Building current development version
If you wish to check out changes not yet made it into a release, you can build the game for yourself.
1. If you dont have it installed already, Install rust and the latest stable toolchain with [rustup.rs](https://rustup.rs/).
2. If you are running Linux, ensure you have [Bevy's dependancies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md) installed.
3. Clone this repository: `git clone git@github.com:fishfolks/punchy.git`
4. In the repository base, `cargo run` to compile and run the project.

## Contributing

Anyone involved in the Fish Folk community must follow our [code of conduct](https://github.com/fishfolk/jumpy/blob/main/CODE_OF_CONDUCT.md).

Punchy is currently at an early stage of development, if you want to contribute the best way to get started is to join us at the [Spicy Lobster Discord](https://discord.gg/4smxjcheE5) and check out our [help-wanted](https://github.com/fishfolk/punchy/labels/help%20wanted) issues.

Before committing and opening a PR, please run the following commands and follow their instructions:

1. `cargo clippy -- -W clippy::correctness -D warnings`
2. `cargo fmt`

Or if you install [`just`](https://github.com/casey/just) you can simply run `just check`.

## MVP Spec

![861A6300-5FFD-4DDC-B4BF-0E8514F4B87C](https://user-images.githubusercontent.com/583842/161247148-0bc07089-1409-48ca-9cc8-ee1a1edddb9e.png)
