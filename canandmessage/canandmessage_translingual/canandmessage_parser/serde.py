import typing
import textwrap
import pprint
NL = "\n"
class SerdeError(Exception):
    pass

class Anything:
    pass

class Serde:
    @classmethod
    def from_dict(cls, data: dict) -> typing.Self:
        egg = cls()
        for field, ftype in typing.get_type_hints(cls).items():
            # 1. check if the field exists in the data.
            if field not in data:
                # if not, we have to either check for Option or a default value.
                if str(ftype).startswith("typing.Optional") or ftype == Anything:
                    setattr(egg, field, None)
                elif hasattr(cls, field):
                    # instantiate the default value if exists otherwise
                    setattr(egg, field, getattr(cls, field)())
                else:
                    print(data)
                    raise SerdeError(f"no attribute {field} found for {cls.__name__}")
                continue
            # 2. we now know the value exists
            # if the ftype is optional we have to unwrap that.
            if str(ftype).startswith("typing.Optional"):
                ftype = ftype.__args__[0]

            if ftype == Anything:
                # anything goes
                setattr(egg, field, data[field])
                continue
            if str(ftype).startswith("typing.List"):
                itype = ftype.__args__[0]
                if issubclass(itype, Serde):
                    setattr(egg, field, [itype.from_dict(v) for v in list(data[field])])
                else:
                    setattr(egg, field, list(data[field]))
            elif str(ftype).startswith("typing.Dict"):
                ktype, itype = ftype.__args__[:2]
                if issubclass(itype, Serde):
                    setattr(egg, field, {ktype(k): itype.from_dict(v) for k, v in data[field].items()})
                else:
                    setattr(egg, field, dict(data[field]))
            elif issubclass(ftype, Serde):
                setattr(egg, field, ftype.from_dict(data[field]))
            else:
                setattr(egg, field, ftype(data[field]))

        return egg
        
    def __repr__(self) -> str:
        return f"""{self.__class__.__name__} {{
{textwrap.indent(("," + NL).join(f"{name}: {pprint.pformat(getattr(self, name))}" 
                                 for name in typing.get_type_hints(self.__class__).keys()), "    ")}
}}"""