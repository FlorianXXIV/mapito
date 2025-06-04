# Mapito
[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![license badge](https://img.shields.io/github/license/FlorianXXIV/modrinth-apitool
)](https://github.com/FlorianXXIV/modrinth-apitool/blob/main/LICENSE)
Mapito (M)odrinth-(api)(to)ol is a Command line application that lets you search
the Modrinth Mod database to download mods or even define your own Modpack.
## Features
- Search modrinth for projects
- Download projects from modrinth
    - Define your own path
- Configurable
- Define your own modpacks
    - Custom name
    - MC versions are easily changed
        - as long as all mods support the version.
    - Add mods by searching through the modrinth database
    - Updating the modpack is just one command away
    - install the pack to a folder of your choice.
## The Custom modpack
Since the modpacks are just textfiles listing all mods and linking to the modfile
on modrinth, you can easily share it with other people by just sending that file
around.
Here is an Example to look at.
[example.mtpck](./example.mtpck)

## Planned
Planned improvements are documented under issues, they are tagged with enhancment
## Install
To install run `cargo install modrinth-apitool`
