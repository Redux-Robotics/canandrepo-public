# The TOML definition system spec V3

Metadata [top_level]
-----------------------------------------

These are top-level TOML keys.

### `name`: str
 - Marketing name of product

### `base`: Array[str]
- Array of base devices to inherit from, prototype-style.
- Multiple inheritance is possible through a "mix-in" system.
- Precedence of data fields goes to later listed things.
- For example, an OTA v1 device would do `["CanandDevice", "OTAv1"]` as the OTAv1 TOML will override the rather vague defaults CanandDevice prescribes for the OTA messages, with the more comprehensive v1 OTA spec.

### `arch`: str
- Architecture of the device
- This value is used to determine things like "how does NVS work" and "what programming language is the firmware written in"
- **Valid architectures are:**
    - `base` for CanandDevice and other not-real devices/mixins
    - `esp32` for ESP32C3 products (OG Canandmag)
    - `rtic` for RTIC-based firmwares (which is most products)
    - `template` for Not-devices

### `is_public`: bool=True
- Specifies whether this TOML file should be processed for public-facing docs at all.
- If false, RST pages won't be generated for the CAN and settings spec.

### `dev_type`: int
- Device type per FRC-CAN. 
- Valid values is from 0-31 inclusive.

### `dev_class`: int
- Device class for FRC-CAN ids.
- All devices so far use 0.
- Valid values are from 0-31 inclusive. This may get dropped. All devices should set this as 0 just in case.

Docs [docs]
-----------------------------------------

### `reverse_msg`: bool=True
Whether or not to reverse the message index order in documentation

### `reverse_stg`: bool=True
Whether or not to reverse the settings index order in documentation

### `reverse_stg_cmd`: bool=True
Whether or not to reverse the setting command index order in documentation

Vendordep [vendordep]
-----------------------------------------

### `java_package`: str
- The base java package of the device in the vendordep.

### `cpp_namespace`: str
- The base cpp namespace of the device in the vendordep.


Messages [msg] table
-----------------------------------------

msg is a table of sub-tables. The struct name is the enum name.
Listed here is the format for a sub-table (e.g. `[msg.STATUS]`)

Message names are by convention `CAPITAL_SNAKE_CASE`. Sensor data outputs are suffixed by `_OUTPUT` by convention.

### `id`: int
The index numer of the message.

### `comment`: str
The comment associated with the protocol message, used as a short summary for documentation.

For longer descriptions used in the CAN device spec make an RST file in the same directory as the toml with the same name and put it in a `-`-delimited heading under the `..can` comment.

### `max_length`: int = None
The maximum count of bytes a message is expected to take up.
Mostly used for signal bit count checking.

### `min_length`: int = 0
The minimum count of bytes a message is expected to take up.
Mostly used for signal bit count checking.

### `length`: int = None
Aliases `max_length = min_length = length`

### `signals`: Array[Signal]
Array of signals. See the signals section for more info.

### `source`: str="device"
Valid values are:
- `device` for device->host packets (e.g. periodic frames)
- `host` for host->device packets (e.g. commands)
- `bidir` for host<->device packets (e.g. OTAv1 data frame)

### `is_public`: bool=True
Whether or not a value is considered "public" and thus shown in publically-facing documentation and vendordep APIs.

### `signals`: Array[Signal]

This is an array of `Signal` sub-tables.

`Signal` subtables
------------------

`Signal`s are sub-fields of both messages and settings. They specify individual values.

Like everything in Redux, it's assumed that the signal (and collections of signals) are little-endian. 

### `name`: str

This is the name of the signal. It is expected to be unique. Codegen systems should typically add a `sig_` prefix to avoid name conflicts and make it obvious this is a signal. 

### `comment`: str

The comment associated with a signal.

### `dtype`: str

The data type of the signal. This determines the bit length of the type. See `Typing` for more information about valid values.

If this signal is a multiplexor, `dtype` _must_ be an enum type. 

### `mux`: bool=False

Specifies if the signal muxes other signals -- that is, specifies the existence of other signals.
**Editor's note:** it is permissible to skip this for proof-of-concept, unless you _really_ want to.

### `muxed_by`: str=None

Specifies the signal that muxes this value. If the string is blank or the key is missing, this is not a muxed value.

### `muxed_match`: Array[Union[str, Table]]=None

Specifices the enum value that the muxing signal must have for this signal to be active.

If this is an array, it must be an array of matching enumers.
If this is an inline table, it must be a contiguous range of values, with `{min=x, max=y}`

### `optional`: bool=False
True if the field doesn't need to strictly exist.
Only valid for signals at the end of messages (NOT settings!)
If you put an optional in the middle of a message, simply _don't_.

Settings [settings] tables
--------------------------

Settings tables are keyed by their name/id pair.

### `id`: uint8_t
This is the setting index number associated with the setting. All signals are essentially filled in with:

```toml
muxed_by = "address" # the setting address field
muxed_match = [id]
```

### `comment`: str

The comment associated with a setting -- used in public docstrings.

### `dtype`: str[Type]

This is the type of the setting's entire value. 
The type can be a primitive numer or a compound type, but has a maximum length of 48 bits.

**The `struct` primitive btype is not allowed here** -- compound types must have an explicit definition.

### `default_value`: Any=None

The default value of the setting.  This has some complex behavior:

* If `readable && writable` is false or the type is `pad`, this setting is ignored.
* If left unset, the default value assigned is left to the implementing type.
 * If the setting is a non-primitive compound type it is left up to each sub-signal.
 * If this type is an `sint`, `uint`, or `float` this is assumed to be 0.
 * For `bitset` this is all 0s (unset)
 * For `buf` this is null (`\x00`) bits.
 * For `enum` this is defined in the enum section definition.

Default values can be specified as such:
* `bit` and `bool` accept boolean literals or the integers 0 and 1
* `sint`, `uint`, and derived types accept, well, _integers_
* `float` and derivatives accept ints or floats
* `bitset` accepts positive integers. You are encouraged to leverage TOML's support for binary literals.
* `buf` accepts positive integers. If you have 6 bytes and want to encode `b"hello"` one can do `0x6f6c6c6568` (little endian assumed. You can solve this by hand by plugging `hex(int.from_bytes(b"Hello", 'little'))` into a python repl.) But like, if you seriously need this feature, ask yourself: **do it stink?**
* `enum` accepts strings with the enum name.

The one drawback of this is if two compound types have needs for different default values, this may be problematic. Oh well. 

### `is_public`: bool=True
Whether the setting is documented at all.

### `vendordep`: bool=True

Whether the setting is in the vendordep's `[name]Details` class.

### `vdep_setting`: bool=True
True if the setting should be listened for in fetch_all_settings and/or is considered settable, False if not. Implied false if `vendordep=False`.

### `readable`: bool=True
True if the setting can be read from. Some settings serve an API function when written to and thus cannot be meaningfully read (e.g. most position sets)

### `writable`: bool=True
True if the setting can be written to. Not all settings are writable, after all (like firmware version).

### `reset_on_default`: bool=True
Specifies if this setting resets to a default value if true.


Primitive Types
---------------
Valid types are:
- `bool`: single bit
- `sint`: signed int
- `uint`: unsigned int
- `float`: floating point (only accepts bits=32 and bits=64)
- `buf`: fixed-size byte buffer
- `bitset`: set of bits
- `pad`: padding bits

`sint`, `uint`, `float`, `buf`, `bitset`, `pad` can all be suffixed with a bit-width to inline-instantiate a derived type with sane defaults -- either for subclassing types, or just directly in signal `type` definitions -- e.g. `sint:16` reppresents a signed integer type with `bits = 16`, a default value of 0, and the typically expected range of values.

Implementations are allowed to only support 32 bits for floats, but are encouraged to support at least 32, 64, and maybe 16.

By default, numeric types default to 0, `bool` defaults to `false`, while `buf`, and `bitset` default to zero values. While `pad` is not supposed to have values, if needed it will always read `0`

Other Types
-----------
- `enum:[name here]`: enums implemented in the `enums` table. 
- `struct`: compound structure with signals, defined in the `types` table.

Special builtin types
---------------------
- `setting_data`: inbuilt struct that is 48 bits long that hold setting data. Can be implemented as a `buf:48` for now.
- `enum:SETTING`: 8-bit enum with each setting name associated with its setting index.
- `enum:SETTING_COMMAND`: 8-bit enum with each setting command name associated with its setting command index.


Types [types] table
-------------------

Types are the base of the whole thing. 
The name of a type is the key to be used in signals.

### `btype`: str

The base type this Type based on.
May be a Primitive, `struct`, or another defined Type (to prototype-extend it), but cannot be an `enum`. For now.

### `utype`: str
A display type hint. Only _really_ relevant for vendordep shenanigans.

### `bits`: int
Number of bits that this type applies to. Mandatory.

### `min`: Numer=None
For numeric types, specifies the minimum allowed value.
If none is specified one is inferred as the smallest encodable value.

For floats this is `-inf`, for a `uint:16` this would be `0` but an `sint:16` woud have `-32768`.
However, you cannot explicitly use `min=nan` or `min=-inf` for float values as the implied bounds should be sufficient.

### `max`: Numer=None
For numeric types, specifies the maximum allowed value.
If none is specified one is inferred from the type and width.

For floats this is `+inf`. However, you cannot explicitly use `max=nan` or `max=+inf`.

### `allow_nan_inf`: bool=True
Whether to allow nan/inf values in float fields

### `default_value`: Any=None

Similar to `default_value` in the settings structure. This _must_ be none for struct types (for now).

The value is interpreted per the `btype` rather than the `dtype` for precision.

### `factor`: Array[Numer]=[1, 1]
A scaling factor to apply for presentation. The first number is the numerator, the second is the denominator. By default this is [1, 1].
Implementors should scream if the second numer is 0.

### `offset`: Numer=0.0
An offset factor to apply for presentation. By default, not applied (1.0)

### `signals`: Array[Signal]=None
Array of signals. Only used in struct-backed types, otherwise ignored.

### `bit_flags`: Array[flag_ent]=None
Mandatory if btype is `bitset`. Sub-tables of name/comment pairs, starting from bit 0 to bit N-1.
Length must be less than or equal to `bits`.

```toml
[types.faults]
btype = "bitset"
bits = 8
bit_flags = [
    { name = "power_cycle", comment = "[power cycle docs]" },
    { name = "can_id_conflict", comment = "[can id conflict docs]" },
    { name = "can_general_error", comment = "[can general error docs]" },
    # .. so on and so forth
]
```

Enums [enums] table
-------------------

Enums can be extended by inheritance by adding more fields in `values` in sub-devices.
They're basically dressing on `uint` types

### `btype`: str[Type] = "uint"
The backing type for the enum. If unspecified, assumes `uint`.
**Note:** This will probably always be uint.

### `bits`: int
The enum width.

### `is_public`: bool=True
Whether the enum is documented at all.

### `default_value`: str[Enum]
The enum key of the default value. One specifies an enum name as string.
Mandatory even if the enum is not used in settings

### `values`: Table

Sub-tables have the keys `id` and `comment`

### Example

```toml
[enums.CLAUSE_JOIN]
btype = "uint"
bits = 2
comment = "Clause joining behavior"
default_value = "kTerminate"
[enums.CLAUSE_JOIN.values]
kTerminate       = {id = 0, comment = "Terminate"}
kOrWithNextSlot  = {id = 1, comment = "Logical OR with next slot"}
kXorWithNextSlot = {id = 2, comment = "Logical XOR with next slot"}
kAndWithNextSlot = {id = 3, comment = "Logical AND with next slot"}
```
