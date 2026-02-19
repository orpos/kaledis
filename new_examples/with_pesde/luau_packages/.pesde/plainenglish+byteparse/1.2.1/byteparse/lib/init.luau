--[[
    plainenglishh/byteparse v1.2.1
   
    Pure-luau library for serialising and deserialising binary formats.
]]

--[[
    MIT License

    Copyright (c) 2025 plainenglish

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
]]

--!native
--!optimize 2
--!strict
--!nolint LocalShadow

local _BYTEPARSE_VERSION = "v1.2.1";

export type u8 = number;
export type u16 = number;
export type u32 = number;
export type i8 = number;
export type i16 = number;
export type i32 = number;
export type f32 = number;
export type f64 = number;
export type varint = number;

export type Endianness = "le" | "be";

export type Serialisable = boolean | number | string | buffer | vector | nil | SerialisableArray | SerialisableTable;
export type SerialisableArray = {Serialisable};
export type SerialisableTable = {[Serialisable]: Serialisable};

export type DataType = "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f32" | "f64";

local TYPE_IDS = {
    NIL = 0,
    BOOLEAN = 1,
    NUMBER = 2,
    VECTOR = 3,
    STRING = 4,
    BUFFER = 5,
    TABLE = 6,  
};

local TABLE_FLAGS = {
    HAS_ARRAY = 0,
    HAS_DICT = 1,
};

local DEFAULT_EXPANSION_RATE = 1.25;

--[=[
    @interface TableParts
    .array {any}
    .hash {[any]: any}
    .array_size number
    .hash_size number
    @private
    @within byteparse
]=]
type TableParts = {
    array: {any},
    hash: {[any]: any},
    array_size: number,
    hash_size: number,
};

local error_mt = {
    __tostring = function(self)
        return `[byteparse ERROR]: {self.reason}: {self.message}`;
    end,
};

local function BPError(reason: string, message: string)
    return setmetatable({ reason = reason, message = message, }, error_mt);
end

--[=[
    Library root.

    @class byteparse
]=]
local byteparse = {};

--[=[
    Extracts the hash and array parts from a table.

    @within byteparse
    @private
    @param tbl {[any]: any}
    @return TableParts
]=]
function byteparse.get_table_parts(tbl: {[any]: any})
    local array_part = {};
    local hash_part = {};
    local next_num = 0;

    local hash_size = 0;
    local array_size = 0;

    for k, v in tbl do
        if type(k) == "number" then
            next_num += 1;
            
            if next_num == k then
                array_size += 1;
                array_part[k] = v;
            else
                hash_part[k] = v;
            end
        else
            hash_size += 1;
            hash_part[k] = v;
        end
    end

    return {
        hash = if hash_size ~= 0 then hash_part else nil,
        array = if array_size ~= 0 then array_part else nil,
        hash_size = hash_size,
        array_size = array_size,
    };
end

--[=[
    Removes the first encountered null byte (and all following data) from a buffer.

    @param buf buffer
    @return buffer
]=]
function byteparse.trim_zbuffer(buf: buffer): buffer
    local cursor = 0;
    for i = 1, buffer.len(buf) do
        if buffer.readu8(buf, cursor) == 0 then
            local new_buf = buffer.create(cursor);
            buffer.copy(new_buf, 0, buf, 0, cursor);
            return new_buf;
        end
        cursor += 1;        
    end
    return buf;
end

--[=[
    Removes the first encountered null byte (and all following data) from a string.

    @param str string
    @return string
]=]
function byteparse.trim_zstring(str: string): string
    return string.gsub(str, "%z.+$", "");
end


--[=[
    Appends a null terminator to the end of a string.
]=]
function byteparse.zterm_string(str: string): string
    return `{str}\0`;
end

--[=[
    Appends a null terminator to the end of a buffer.
]=]
function byteparse.zterm_buffer(buf: buffer): buffer
    local new_buf = buffer.create(buffer.len(buf) + 1);
    buffer.copy(new_buf, 0, buf, 0);
    return new_buf;
end

--[=[
    Creates a new buffer from a hexadecimal string, ignoring any characters that
    aren't [0-9A-Fa-f].
]=]
function byteparse.buffer_from_hex(hex: string)
    local hex = string.gsub(hex, "%W", "");
    local writer = byteparse.writer(0);
    
    for i = 1, #hex, 2  do
        local byte = tonumber(string.sub(hex, i, i + 1), 16) :: number;
        writer.write_u8(byte);
    end

    return writer.to_buffer();
end

--[=[
    Reverses a range of values in a buffer in place.
]=]
function byteparse.buffer_reverse(b: buffer, offset: number?, count: number?)
    local offset = offset or 0;
    local count = (count or buffer.len(b));

    if count % 2 ~= 0 then
        count += 1;
    end

    local temp;

    for i = offset, offset + count//2 - 1 do
        temp = buffer.readu8(b, offset + (count - 1 - i));
        buffer.writeu8(b, offset + (count - 1 - i), buffer.readu8(b, i));
        buffer.writeu8(b, i, temp);
    end
end

--[=[
    The library version as a semver string. E.g `v1.0.0`

    @within byteparse
    @prop version string
]=]
byteparse.version = _BYTEPARSE_VERSION;

--[=[
    Creates a writer instance with a given starting size and expansion rate.

    @param expansion_rate number? -- Multiplier used to resize the internal buffer when attempting to write more data than the internal buffer can take.
    @return writer
]=]
function byteparse.writer(start_size: number, expansion_rate: number?)
    local expansion_rate = expansion_rate or DEFAULT_EXPANSION_RATE;

    local b = buffer.create(start_size);
    local cursor = 0;
    local written_size = 0;
    local default_vector_size = 3;

    --[=[
        Writer instance used to serialise to binary formats.
        @class writer
    ]=]
    local writer = {};

    --[=[
        Resizes the internal buffer to a given size.

        @within writer
    ]=]
    local function set_capacity(new_size: number)
        local new_b = buffer.create(new_size);
        buffer.copy(new_b, 0, b);
        b = new_b;
    end

    --[=[
        Ensures the internal buffer can handle `extra_size` more data by 
        expanding the capacity if necessary.

        Resizes relative to the cursor.

        @within writer
    ]=]
    local function expand(extra_size: number)
        local size = buffer.len(b);
        if cursor + extra_size > size then
            set_capacity((size + extra_size) * expansion_rate);
        end 
    end

    --[=[
        Advances the write cursor by a set amount following a write.
        Updates the written_size accordingly.

        @within writer
        @ignore
    ]=]
    local function advance_write(offset: number)
        cursor += offset;

        if cursor > written_size then
            written_size = cursor;
        end
    end

    --[=[
        Sets the write cursor to a given position.

        @within writer
    ]=]
    local function set_cursor(position: number)
        cursor = position;
    end

    --[=[
        Returns the current cursor position.

        @within writer
    ]=]
    local function get_cursor(): number
        return cursor;
    end

    --[=[
        Returns the current written size.

        @within writer
    ]=]
    local function get_size(): number
        return written_size;
    end

    --[=[
        Returns the internal buffer capacity.

        @within writer
    ]=]
    local function get_capacity(): number
        return buffer.len(b);
    end

    writer.set_capacity = set_capacity;
    writer.expand = expand;
    writer.set_cursor = set_cursor;
    writer.get_cursor = get_cursor;
    writer.get_capacity = get_capacity;
    writer.get_size = get_size;

    --[=[
        Sets the vector size. The given vector size cannot exceed the VMs
        'LUA_VECTOR_SIZE' size.

        @error InvalidVecSize -- Errors when passed a vector size that isn't 3 or 4.
    ]=]
    function writer.set_default_vector_size(vec_size: number)
        if vec_size ~= 3 and vec_size ~= 4 then
            error(BPError("InvalidVecSize", `expected either 3 or 4, got {vec_size}`), 2);
        end

        default_vector_size = vec_size;
    end

    --[=[
        Returns a buffer containing the written data.
    ]=]
    function writer.to_buffer(): buffer
        local out_buf = buffer.create(written_size);
        buffer.copy(out_buf, 0, b, 0, written_size);
        return out_buf;
    end

    --[=[
        Returns a string containing the written data.
    ]=]
    function writer.to_string(): string
        return buffer.tostring(writer.to_buffer());
    end

    --[=[
        Writes an unsigned 8 bit integer. Wraps around if provided a number
        larger than 8 bits.
    ]=]
    function writer.write_u8(value: u8)
        expand(1);
        buffer.writeu8(b, cursor, value);
        advance_write(1);
    end

    --[=[
        Writes an unsigned 16 bit integer. Wraps around if provided a number
        larger than 16 bits.
    ]=]
    function writer.write_u16(value: u16)
        expand(2);
        buffer.writeu16(b, cursor, value);
        advance_write(2);
    end

    --[=[
        Writes an unsigned 32 bit integer. Wraps around if provided a number
        larger than 32 bits.
    ]=]
    function writer.write_u32(value: u32) 
        expand(4);
        buffer.writeu32(b, cursor, value);
        advance_write(4);
    end

    --[=[
        Writes an unsigned 8 bit integer. Wraps around if provided a number
        larger than 8 bits.
    ]=]
    function writer.write_i8(value: i8)
        expand(1);
        buffer.writei8(b, cursor, value);
        advance_write(1);
    end

    --[=[
        Writes an unsigned 16 bit integer. Wraps around if provided a number
        larger than 16 bits.
    ]=]
    function writer.write_i16(value: i16)
        expand(2);
        buffer.writei16(b, cursor, value);
        advance_write(2);
    end

    --[=[
        Writes an unsigned 32 bit integer. Wraps around if provided a number
        larger than 32 bits.
    ]=]
    function writer.write_i32(value: i32) 
        expand(4);
        buffer.writei32(b, cursor, value);
        advance_write(4);
    end

    --[=[
        Writes a 32 bit floating point number. Wraps around if provided a number
        larger than 32 bits.
    ]=]
    function writer.write_f32(value: f32)
        expand(4);
        buffer.writef32(b, cursor, value);
        advance_write(4);
    end

    --[=[
        Writes a 64 bit floating point number.
    ]=]
    function writer.write_f64(value: f64) 
        expand(8);
        buffer.writef64(b, cursor, value);
        advance_write(8);
    end

    --[=[
        Writes a ProtoBuf [Base 128 Variable-width integer](https://protobuf.dev/programming-guides/encoding/#varints).
    ]=]
    function writer.write_varint(value: varint) 
        local n = value;

        while n >= 0x80 do
            writer.write_u8(bit32.bor(0b1000_0000, n));
            n = bit32.rshift(n, 7);
        end
        
        writer.write_u8(n);
    end

    --[=[
        Writes a boolean.
    ]=]
    function writer.write_bool(value: boolean)
        writer.write_u8(if value then 1 else 0);
    end

    --[=[
        Writes a null byte.
    ]=]
    function writer.write_null()
        writer.write_u8(0);
    end

    --[=[
        Writes a string without a prefixed length, optionally trimmed/padded to 
        a given size.

        Note: You should ensure the binary format has a way of encoding the
        strings size, as this function writes the string raw and doesn't write
        any data a reader can use to determine the size.
    ]=]
    function writer.write_string(value: string, size: number?)
        local size = size or #value;
        local real_size = math.min(#value, size);

        expand(size);
        buffer.fill(b, cursor, 0, size);
        buffer.writestring(b, cursor, value, real_size);
        --[[if size > #value then
            local size_remaining = size - real_size;
        end]]

        advance_write(size);
    end

    --[=[
        Writes a length prefixed string.
    ]=]
    function writer.write_lstring(value: string, size: number?)
        local size = size or #value;
        writer.write_varint(size);
        writer.write_string(value, size);
    end

    --[=[
        Writes a null terminated (C Style) string. 
        
        When passed `size`, trims the string to `size` or pads the end with zeros.
        The `size` parameters specifies the length of the string buffer, and
        includes the null-terminator.

        :::warning
        This function doesn't check whether the passed value is null-safe, and
        will write the entire string regardless of embedded null bytes.

        Consider wrapping the passed value with `byteparse.trim_zstring` if you
        can't be sure there are no embedded null bytes.
        :::
    ]=]
    function writer.write_zstring(value: string, size: number?)
        if size then
            value = string.sub(value, 1, size - 1);
        end

        writer.write_string(byteparse.zterm_string(value), size);
    end

    --[=[
        Writes a buffer without a prefixed length, optionally trimmed or null
        padded to to a specified size.

        Note: You should ensure the binary format has a way of encoding the
        buffers size, as this function writes the buffer raw and doesn't write
        any data a reader can use to determine the size.
    ]=]
    function writer.write_buffer(value: buffer, offset: number?, size: number?)
        local size = size or buffer.len(value);
        local real_size = math.min(buffer.len(value), size) - (offset or 0);
        expand(size);
        buffer.copy(b, cursor, value, offset, real_size);
        
        if size > buffer.len(value) then
            local size_remaining = size - real_size;
            buffer.fill(b, cursor + real_size, 0, size_remaining);
            advance_write(size_remaining);
        end

        advance_write(size);
    end

    --[=[
        Writes a length prefixed buffer.
    ]=]
    function writer.write_lbuffer(value: buffer, offset: number?, size: number?)
        local size = size or buffer.len(value);
        writer.write_varint(size);
        writer.write_buffer(value, offset, size);
    end

    --[=[
        Writes a null terminated (C Style) buffer.
        
        Assumes the buffer is null safe.

        :::warning
        This function doesn't check whether the passed value is null-safe, and
        will write the entire buffer regardless of embedded null bytes.

        Consider wrapping the passed value with `byteparse.trim_zbuffer` if you
        can't be sure there are no embedded null bytes.
        :::
    ]=]
    function writer.write_zbuffer(value: buffer, offset: number?, size: number?)
        writer.write_buffer(byteparse.zterm_buffer(value), offset, size);
    end

    --[=[
        Writes a array without a prefixed length.

        Assumes the writer_callback function advances the cursor.

        Note: You should ensure the binary format has a way of encoding the
        arrays size, as this function doesn't write any data a reader can use
        to determine the size.

        @param ... B... -- Additional parameters to pass to the writer_callback.
    ]=]
    function writer.write_array<T, B...>(value: {T}, writer_callback: (T) -> (), ...: B...)
        for _, v in value do
            writer_callback(v, ...);
        end
    end

    --[=[
        Writes a length prefixed array.

        Assumes the writer_callback function advances the cursor.

        @param ... B... -- Additional parameters to pass to the writer_callback.
    ]=]
    function writer.write_larray<T, B...>(value: {T}, writer_callback: (T) -> (), ...: B...)
        writer.write_varint(#value);
        writer.write_array(value, writer_callback, ...);
    end

    --[=[
        Writes a null terminated array.

        Assumes the writer_callback function advances the cursor. The writer_callback function
        can freely emit null bytes **after** the first byte, provided it advances the cursor properly.

        @param ... B... -- Additional parameters to pass to the writer_callback.
    ]=]
    function writer.write_zarray<T, B...>(value: {T}, writer_callback: (T) -> (), ...: B...)
        writer.write_array(value, writer_callback, ...);
        writer.write_null();
    end

    --[=[
        Writes a vector.
    ]=]
    function writer.write_vector(value: vector, vector_size: number?)
        writer.write_f32(value.x);
        writer.write_f32(value.y);
        writer.write_f32(value.z);
        
        if (vector_size or default_vector_size) == 4 then
            writer.write_f32((value :: any).w);
        end
    end

    --[=[
        Writes a serialisable table.
    ]=]
    function writer.write_table(value: SerialisableTable)
        local parts = byteparse.get_table_parts(value);

        local flags = 0;

        if parts.array then
            flags = bit32.replace(flags, 1, TABLE_FLAGS.HAS_ARRAY, 1);
        end
        
        if parts.hash then
            flags = bit32.replace(flags, 1, TABLE_FLAGS.HAS_DICT, 1);
        end

        writer.write_u8(flags);

        if parts.array then
            writer.write_varint(parts.array_size);
            writer.write_array(parts.array, writer.write_any);
        end

        if parts.hash then
            writer.write_varint(parts.hash_size);
            for k, v in parts.hash do
                writer.write_any(k);
                writer.write_any(v);
            end
        end
    end

    --[=[
        Writes any serialisable type, with the Luau TValue prefixed.
    ]=]
    function writer.write_any(value: Serialisable)        
        if type(value) == "nil" then
            writer.write_u8(TYPE_IDS.NIL);
        elseif type(value) == "boolean" then
            writer.write_u8(TYPE_IDS.BOOLEAN);
            writer.write_bool(value);
        elseif type(value) == "number" then
            writer.write_u8(TYPE_IDS.NUMBER);
            writer.write_f64(value);
        elseif type(value) == "vector" or (type(value) == "table" and getmetatable(value) and getmetatable(value).__type == "vector") then
            writer.write_u8(TYPE_IDS.VECTOR);
            writer.write_vector(value :: any);
        elseif type(value) == "string" then
            writer.write_u8(TYPE_IDS.STRING);
            writer.write_lstring(value);
        elseif type(value) == "buffer" then
            writer.write_u8(TYPE_IDS.BUFFER);
            writer.write_lbuffer(value);
        elseif type(value) == "table" then
            writer.write_u8(TYPE_IDS.TABLE);
            writer.write_table(value);
        end
    end

    --[=[
        Writes an optional value.

        This function writes a boolean indicating whether the value is present,
        followed by the data itself (if present) using `writer_callback`.

        #### Example Use
        ```lua
        writer.write_optional(1, writer.write_u8); -- Writes `0101`
        writer.write_optional(nil, writer.write_u8); -- Writes `00`
        ```
    ]=]
    function writer.write_optional<T, B...>(value: T?, writer_callback: (T) -> (), ...: B...)
        writer.write_bool(value ~= nil);
        if value ~= nil then
            writer_callback(value, ...);
        end
    end

    return writer;
end

--[=[
    Creates a reader instance from a buffer.

    @return reader -- The new reader instance.
]=]
function byteparse.reader(b: buffer)
    local cursor = 0;
    local default_vector_size = 3;

    --[=[
        Advances the read cursor by a set amount following a read.

        @within reader
        @ignore
    ]=]
    local function advance(offset: number)
        cursor += offset;
    end

    --[=[
        Sets the read cursor to a set position.

        @within reader
    ]=]
    local function set_cursor(position: number)
        cursor = position;
    end

    --[=[
        Returns the current cursor position.

        @within reader
    ]=]
    local function get_cursor(): number
        return cursor;
    end

    --[=[
        Reader instance used to deserialise to binary formats.

        @class reader
    ]=]
    local reader = {};

    reader.set_cursor = set_cursor;
    reader.get_cursor = get_cursor;

    --[=[
        Sets the vector size. The given vector size cannot exceed the VMs
        'LUA_VECTOR_SIZE' size.

        @error InvalidVecSize -- Errors when passed a vector size that isn't 3 or 4.
    ]=]
    function reader.set_default_vector_size(vec_size: number)
        if vec_size ~= 3 and vec_size ~= 4 then
            error(BPError("InvalidVecSize", `expected either 3 or 4, got {vec_size}`), 2);
        end

        default_vector_size = vec_size;
    end

    --[=[
        Reads an unsigned 8 bit integer.
    ]=]
    function reader.read_u8(): u8
        local value = buffer.readu8(b, cursor);
        advance(1);
        return value;
    end

    --[=[
        Reads an unsigned 16 bit integer.
    ]=]
    function reader.read_u16(): u16
        local value = buffer.readu16(b, cursor);
        advance(2);
        return value;
    end

    --[=[
        Reads an unsigned 32 bit integer.
    ]=]
    function reader.read_u32(): u32
        local value = buffer.readu32(b, cursor);
        advance(4);
        return value;
    end

    --[=[
        Reads a signed 8 bit integer.
    ]=]
    function reader.read_i8(): i8
        local value = buffer.readi8(b, cursor);
        advance(1);
        return value;
    end

    --[=[
        Reads a signed 16 bit integer.
    ]=]
    function reader.read_i16(): i16
        local value = buffer.readi16(b, cursor);
        advance(2);
        return value;
    end

    --[=[
        Reads a signed 32 bit integer.
    ]=]
    function reader.read_i32(): i32
        local value = buffer.readi32(b, cursor);
        advance(4);
        return value;
    end

    --[=[
        Reads a 32 bit floating point number.
    ]=]
    function reader.read_f32(): f32
        local value = buffer.readf32(b, cursor);
        advance(4);
        return value;
    end

    --[=[
        Reads a 64 bit floating point number.
    ]=]
    function reader.read_f64(): f64
        local value = buffer.readf64(b, cursor);
        advance(8);
        return value;
    end

    --[=[
        Reads a ProtoBuf [Base 128 Variable-width integer](https://protobuf.dev/programming-guides/encoding/#varints).
    ]=]
    function reader.read_varint(): varint 
        -- Inspired by fiu's implementation
        local result = 0;
		for i = 0, 4 do
			local value = reader.read_u8();
			result = bit32.bor(result, bit32.lshift(bit32.band(value, 0x7F), i * 7));
			if not bit32.btest(value, 0x80) then
				break;
			end
		end
		return result;
    end

    --[=[
        Reads a boolean.
    ]=]
    function reader.read_bool(): boolean
        return reader.read_u8() == 1;
    end

    --[=[
        Reads a string.
    ]=]
    function reader.read_string(length: number): string
        local value = buffer.readstring(b, cursor, length);
        advance(length);
        return value;
    end

    --[=[
        Reads a length prefixed string.
    ]=]
    function reader.read_lstring(): string
        local length = reader.read_varint();
        return reader.read_string(length);
    end

    --[=[
        Reads a null terminated string.

        If `size` is provided, this function will consume the entire string
        buffer, disregarding data past the first null byte. If no null byte is
        encountered when the entire `size` has been read, it returns what has
        been already been read and exits.
    ]=]
    function reader.read_zstring(size: number?): string
        local start = cursor;
        while true do
            local byte = reader.read_u8();
            local length = cursor - start;

            if byte == 0 or (size and size == length) then
                if size then
                    local remaining_space = size - length;
                    if remaining_space > 0 then
                        advance(remaining_space);
                    end
                end
                
                return buffer.readstring(b, start, length - 1);
            else
                continue;
            end
        end
    end

    --[=[
        Reads a buffer.
    ]=]
    function reader.read_buffer(length: number): buffer
        local value = buffer.create(length);
        buffer.copy(value, 0, b, cursor, length);
        advance(length);
        return value;
    end

    --[=[
        Reads a length prefixed buffer.
    ]=]
    function reader.read_lbuffer(): buffer
        local length = reader.read_varint();
        return reader.read_buffer(length);
    end

    --[=[
        Reads a null terminated buffer.
    ]=]
    function reader.read_zbuffer(): buffer
        local start = cursor;
        while true do
            if reader.read_u8() == 0 then
                local length = (cursor - 1) - start;
                local value = buffer.create(length);
                buffer.copy(value, 0, b, start, length);
                return value;
            else
                continue;
            end
        end
    end

    --[=[
        Reads an array of values.

        Assumes `reader_callback` advances the cursor.
        Stops reading if the reader_callback returns nil.

        @param ... B... -- Additional parameters to pass to the reader_callback.
    ]=]
    function reader.read_array<T, B...>(size: number, reader_callback: (B...) -> T?, ...: B...): {T}
        local values = {};
        if size == -1 then
            while true do
                local next_value = reader_callback(...);
                if next_value then
                    table.insert(values, next_value);
                else
                    return values;
                end
            end
        else
            for i = 1, size do
                local next_value = reader_callback(...);
                if next_value then
                    table.insert(values, next_value);
                else
                    return values;
                end
            end
        end

        return values;
    end

    --[=[
        Reads a length prefixed array of values.

        Assumes `reader_callback` advances the cursor.
        Stops reading if the reader_callback returns nil.

        @param ... B... -- Additional parameters to pass to the reader_callback.
    ]=]
    function reader.read_larray<T, B...>(reader_callback: (B...) -> T?, ...: B...): {T}
        local size = reader.read_varint();
        return reader.read_array(size, reader_callback, ...);
    end

    --[=[
        Reads a null terminated array of values.

        Assumes `reader_callback` advances the cursor.
        Stops reading if the next byte is `0`, or if the reader_callback returns nil.

        @param ... B... -- Additional parameters to pass to the reader_callback.
    ]=]
    function reader.read_zarray<T, B...>(reader_callback: (B...) -> T?, ...: B...): {T}
        local values = {};
        while true do
            if buffer.readu8(b, cursor) == 0 then
                advance(1);
                return values;
            end

            local next_value = reader_callback(...);
            if next_value then
                table.insert(values, next_value);
            else
                return values;
            end
        end
    end

    --[=[
        Reads a vector.

        This function reads either three or four (depending on vector_size)
    ]=]
    function reader.read_vector(vector_size: number?): vector
        local x = reader.read_f32();
        local y = reader.read_f32();
        local z = reader.read_f32();
        
        if (vector_size or default_vector_size) == 4 then
            local w = reader.read_f32();
            return (vector.create :: any)(x, y, z, w);
        end

        return vector.create(x, y, z);
    end

    --[=[
        Reads a table.
    ]=]
    function reader.read_table(): SerialisableTable
        local flags = reader.read_u8();
        local has_array = bit32.extract(flags, TABLE_FLAGS.HAS_ARRAY) == 1;
        local has_dict = bit32.extract(flags, TABLE_FLAGS.HAS_DICT) == 1;
        
        local values: {[Serialisable]: Serialisable} = {};

        if has_array then
            local size = reader.read_varint();
            for i = 1, size do
                values[i] = reader.read_any();
            end
        end

        if has_dict then
            local size = reader.read_varint();
            for i = 1, size do
                local k = reader.read_any();
                local v = reader.read_any();

                values[k] = v;
            end
        end

        return values;
    end

    --[=[
        Reads a type-prefixed value.

        @error InvalidTypeID -- Errors when an unknown type id is read.
    ]=]
    function reader.read_any(): Serialisable
        local type_id = reader.read_u8();

        if type_id == TYPE_IDS.NIL then
            return nil;
        elseif type_id == TYPE_IDS.BOOLEAN then
            return reader.read_bool();
        elseif type_id == TYPE_IDS.NUMBER then
            return reader.read_f64();
        elseif type_id == TYPE_IDS.VECTOR then
            return reader.read_vector();
        elseif type_id == TYPE_IDS.STRING then
            return reader.read_lstring();
        elseif type_id == TYPE_IDS.BUFFER then
            return reader.read_lbuffer();
        elseif type_id == TYPE_IDS.TABLE then
            return reader.read_table();
        end

        error(BPError("InvalidTypeID", `got invalid type id {type_id}`));
    end

    --[=[
        Reads an optional value.

        This function reads a potentially absent value by reading a boolean to
        determine whether the value is present, and reading the value with
        `reader_callback` if it is.

        #### Example
        ```lua
        local reader = byteparse.reader(byteparse.buffer_from_hex("0101 00"));
        reader.read_optional(reader.read_u8); -- Returns `1`
        reader.read_optional(reader.read_u8); -- Returns `nil`
        ```
    ]=]
    function reader.read_optional<T, B...>(reader_callback: (B...) -> T, ...: B...): T?
        if reader.read_bool() then
            return reader_callback(...);
        end

        return;
    end

    return reader;
end

return byteparse;
