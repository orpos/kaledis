# Dal
Dal(달) is a Luau-to-Lua transpiler based on `darklua`, designed specifically for `Lua 5.3`.

## Note
This project is still in W.I.P

## TO-DOs
- [x] Implement CLI.
- [x] Implement basic transpilation process using `darklua` and `full-moon`.
- [x] Implement modifiers (such as converting number literals and generalized iterations)
- [x] Implement basic lua polyfills.
- [x] Add tests for polyfills.
- [ ] Add tests for transpilation. (to ensure the same results in lua and luau)
- [ ] Add tests for dal internally.
- [x] Add logging for dal internally for debug.
- [x] `convert_bit32` modifier now converts `bit32.btest`.
- [x] Add comments for docs and code readability. (WIP)
- [x] Optimize polyfill.

## Installation
Coming soon! (will be available at `rokit` and `crates.io`(for `cargo install`))

## Usage

### `init`
Initializes dal manifest file in the current path.
```sh
dal init
```

### `fetch`
Fetches and updates lua polyfills.
* This polyfill can be found [here](https://github.com/CavefulGames/dal-polyfill).
```sh
dal fetch
```

### `transpile`
Transpiles luau code to lua code.
```sh
dal transpile [input] [output]
```

## Example
### `dal.toml`
```toml
file_extension = "lua"
target_version = "lua53"
minify = true

[modifiers]
convert_bit32 = true

[globals]

```

### `inputs/input.luau`
```luau
local log = math.log
local floor = math.floor
local x = bit32
local band = x.band
local rshift = x.rshift
local lshift = x.lshift
local bnot = x.bnot
local bor = x.bor
local t = table.create(1)

local function byteswap(n: number): number
	return bor(bor(bor(lshift(n, 24), band(lshift(n, 8), 0xff0000)), band(rshift(n, 8), 0xff00)), rshift(n, 24))
end

print(byteswap(5))
print(log(5))
print(floor(0.5))
print(t)

```

### `outputs/output.luau`
```lua
local math=require'./__polyfill__'.math local table=require'./__polyfill__'.table local io=nil local module=nil local package=nil local dofile=nil local loadfile=nil local load=nil local log=math.log
local floor=math.floor
 do
end  do
end  do
end  do  end  do

end  do
end local t=table.create(1)

local function byteswap(n)
return ((((((((n<<24)&0xFFFFFFFF)|((((n<<8)&0xFFFFFFFF)&0xff0000)&0xFFFFFFFF))&0xFFFFFFFF)|((((n>>8)&0xFFFFFFFF)&0xff00)&0xFFFFFFFF))&0xFFFFFFFF)|((n>>24)&0xFFFFFFFF))&0xFFFFFFFF)
end

print(byteswap(5))
print(log(5))
print(floor(0.5))
print(t)
```

## Special Thanks
- [seaofvoices/darklua](https://github.com/seaofvoices/darklua) - Providing important and cool lua mutating rules.
- [Kampfkarren/full-moon](https://github.com/Kampfkarren/full-moon) - A lossless Lua parser.

## Trivia
The name of this project, Dal, translates to "moon" in Korean.
