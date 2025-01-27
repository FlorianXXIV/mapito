# modrinth-apitool
A tool to interact with the modrinth api using rust reqwest
## Features
- Search modrinth for projects
- Download projects from modrinth
    - With custom download Path
    - Can download latest version
    - Can download latest version for specific mc version
## Planned
- Better Download options
    - List available versions by MC version
    - download specific mod version
    - automatically download dependencies
        - avoid multiple downloads
- Be able add mods to a modlist file to automatically download the latest version of mods
- ...
## Install
To install run `cargo install modrinth-apitool`
