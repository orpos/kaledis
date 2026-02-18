# Byteparse

A single-file pure-luau library for serialising and deserialising binary formats
with buffers.

## Example Usage

```luau
local byteparse = require("../byteparse");

local writer = byteparse.writer(100);

writer.write_u8(255);
writer.write_i32(-10000);
writer.write_table({
    a = 1,
    b = 2,
});

local reader = byteparse.reader(writer.to_buffer());

print(reader.read_u8()); --> 255
print(reader.read_i32()); --> -10000
print(reader.read_table()); --> { a = 1, b = 2 }
```

## Installation

### Pesde

To install using pesde, run the following command in the project root:

```
pesde add plainenglish/byteparse
```

### Vendored

Simply copy `lib/init.luau` into your project.

If you get type errors due to differing project setups, just add `--!nolint`
and `--!nocheck` to the top of the file as required.
