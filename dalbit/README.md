# [Dalbit](https://crates.io/crates/dalbit)

#### Why is this folder here? it's because we use custom rules to provide some features for example module resolution

<a href="https://discord.gg/ATVVsNNv3u"><img alt="Discord" src="https://img.shields.io/discord/385151591524597761?style=plastic&logo=discord&color=%235865F2" /></a>

Dalbit(달빛) is a Luau-to-Lua transpiler, designed specifically for `Lua 5.3`.

## TO-DOs
- [x] Implement CLI.
- [x] Implement basic transpilation process using `darklua` and `full-moon`.
- [x] Implement modifiers (such as converting number literals and generalized iterations)
- [x] Implement basic lua polyfills.
- [x] Add tests for polyfills.
- [ ] Add tests for transpilation. (to ensure the same results in lua and luau)
- [ ] Add tests for dalbit internally.
- [x] Add logging for dalbit internally for debug.
- [x] `convert_bit32` modifier now converts `bit32.btest`.
- [x] Add comments for docs and code readability. (WIP)
- [x] Optimize polyfill.

## Installation

### [From Releases](https://github.com/CavefulGames/dalbit/releases)

### Using Cargo (build from source)
```sh
cargo install dalbit --locked
```

## Usage

### `init`
Initializes dalbit manifest file in the current path.
```sh
dalbit init
```

### `fetch`
Fetches and updates lua polyfills.
* This polyfill can be found [here](https://github.com/CavefulGames/dalbit-polyfill).
```sh
dalbit fetch
```

### `transpile`
Transpiles luau code to lua code.
```sh
dalbit transpile
```

### `clean`
Cleans polyfill caches from disk.
```sh
dalbit clean
```

## Example
### `dalbit.toml`
```toml
input = "input.luau"
output = "output.lua"
file_extension = "lua"
target_version = "lua53"
minify = true

[modifiers]

[polyfill]
repository = "https://github.com/CavefulGames/dalbit-polyfill"
injection_path = "__polyfill__"

```

### `inputs/input.luau`
```luau
local obj = { items = {1, 4, 9} }
setmetatable(obj, { __iter = function(o) return next, o.items end })

for k, v in obj do
    print(k * k)
end

```

### `outputs/output.luau`
```lua
local setmetatable=require'./__polyfill__'.setmetatable local __DALBIT_getmetatable_iter=require'./__polyfill__'.__DALBIT_getmetatable_iter local type=require'./__polyfill__'.type local next=require'./__polyfill__'.next local io=nil local module=nil local package=nil local dofile=nil local loadfile=nil local load=nil local obj={items={1,4,9}}
setmetatable(obj,{__iter=function(o)return next,o.items end})do local _DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_invare234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_controle234e8bef135bb4c=

obj if type(_DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c)=='table'then local m=__DALBIT_getmetatable_iter(_DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c)if type(m)=='table'and type(m.__iter)=='function'then _DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_invare234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_controle234e8bef135bb4c=m.__iter(_DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c)else _DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_invare234e8bef135bb4c, _DALBIT_REMOVE_GENERALIZED_ITERATION_controle234e8bef135bb4c=next, _DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c end end for k,v in _DALBIT_REMOVE_GENERALIZED_ITERATION_itere234e8bef135bb4c,_DALBIT_REMOVE_GENERALIZED_ITERATION_invare234e8bef135bb4c,_DALBIT_REMOVE_GENERALIZED_ITERATION_controle234e8bef135bb4c do
print(k*k)
end end
```

## How does it work?
- Dalbit utilizes darklua and full-moon to transform lua scripts.

## Real-world use cases
- [Kaledis](https://github.com/orpos/kaledis) - A tool that enables Luau to work with Love2D, simplifying project management, transpiling, and configuration.
- Overblox - A tool that can transpile Roblox scripts to OVERDARE scripts using Dalbit.

## Why `darklua-demo` over `darklua`?
- `darklua-demo` is a temporary fork to work properly with dal.
- `darklua-demo` will be replaced by official `darklua` once darklua released with important features to work properly with dal.

## Contributions
Any issues, advices, and PRs for contribution are welcome!

## Special Thanks
- [seaofvoices/darklua](https://github.com/seaofvoices/darklua) - Providing important and cool lua mutating rules.
- [Kampfkarren/full-moon](https://github.com/Kampfkarren/full-moon) - A lossless Lua parser.

## Trivia
The name of this project, Dalbit, translates to "moonshine" in Korean.
