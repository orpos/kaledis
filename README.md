<div align="center">
<img width=90 height=90 src="./images/image.png" align="center" />

# Kaledis

</div>

Kaledis is a tool for allowing Luau to be used with Love2D via transpiling, alongside providing easier & simpler management of Love2D projects.

It has many resources to make your life much easier when using Love2D:
* Transpiles Luau into compatible Love2D code, allowing type annotations, libraries and other features to be implemented.
* Automatically manages and provides Love2D installations.
* Simple commands and CLI, you'll get the hang of it in no time.
* Easily create & ship your project to the current OS you build the project in.
* A more friendly frontend configuration framework, using a TOML file instead of a *conf.lua*
  * If you need to make it dynamic, we allow you to still use a *conf.lua* file.

## Installation
*Note: The only available builds are for Windows. MacOS and Linux builds have not been tested.*

### From Releases
Go to the Releases page and download the `kaledis.exe` file. *Windows only.*

### From Source
Clone the repo, then use `cargo build` to build the project from scratch *Probably all platforms.*

## Credits
- [Dal](https://github.com/CavefulGames/dal) for the awesome transpiling system.

## FAQ
### Why the name 'Kaledis'?
Kaledis in latin means "moons" or "more than 1 moon", and by the fact that Luau and Love2D are "incompatible" and the package solves that problem, it was given this name.

### Who I contact for source code related stuff?
If you need anything regarding the code, you can contact lettuce-magician and he will forward the topic to ordep (that actually edits the code).

### Why are the type definition files so ugly and are lacking some features?
Luau LSP's typedefs file parsing is kinda weird, and sometimes it crashes or memory leaks. Leaving the only option to weird workarounds.
Currently we're waiting for the [environments](https://github.com/JohnnyMorganz/luau-lsp/pull/84) feature to release so we can finally have proper type definitions for the Love2D environment.