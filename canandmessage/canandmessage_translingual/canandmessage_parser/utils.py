import typing

def none_map(v, f):
    if v is None:
        return None
    return f(v)

def unwrap_or(v, d):
    if v is None:
        return d
    return v

def unwrap_or_else(v, f):
    if v is None:
        return f()
    return v
    

def panic(e: Exception):
    raise e

def read_suffix(s: str) -> str:
    parts = s.split(":")
    if len(parts) < 2:
        panic("where's the suffix dawg in \"{s}\"")
    return parts[1]

def read_suffix_as_usize(s: str) -> int:
    parts = s.split(":")
    if len(parts) < 2:
        panic("hey dumbass you forgot the bit length in \"{s}\"")
    return int(parts[1])

def default_uint_max(width: int) -> int:
    return (1 << width) - 1

def default_sint_min(width: int) -> int:
    return -(1 << (width - 1))

def default_sint_max(width: int) -> int:
    return (1 << (width - 1))-1

def decode_bounds_f64(min, max) -> typing.Tuple[float | None, float | None]:
    return none_map(min, float), none_map(max, float)

def opt_value_to_opt_u64(v) -> int | None:
    return none_map(v, int)

def opt_value_to_opt_i64(v) -> int | None:
    return none_map(v, int)

def opt_value_to_opt_f64(v) -> float | None:
    return none_map(v, float)

def opt_value_to_opt_bool(v) -> bool | None:
    return none_map(v, bool)

def screaming_snake_to_kamel(s: str) -> str:
    return "k" + "".join(c.capitalize() for c in s.split("_"))

def screaming_snake_to_camel(s: str) -> str:
    return "".join(c.capitalize() for c in s.split("_"))

def snake_to_stilted_camel(s: str) -> str:
    v = screaming_snake_to_camel(s)
    if len(v) < 2:
        return v.lower()
    return v[0].lower() + v[1:]

def rsort_by_ent_id(dct: dict):
    return sorted(dct.items(), key=lambda b: b[1].id, reverse=True)

def uwidth(width):
    if width <= 8: 
        return 8
    if width <= 16: 
        return 16
    if width <= 32: 
        return 32
    else:
        return 64

def padder_fn(lstr: typing.List[str]):
    v = len(max(lstr, key=len))
    return lambda s: s.ljust(v)