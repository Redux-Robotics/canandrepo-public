
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *


@dataclasses.dataclass
class EnumerateRequest(BaseMessage):
    """Enumerate request"""
    __meta__ = MessageMeta(device_type=0, id=0, min_length=0, max_length=8)




@dataclasses.dataclass
class TimesyncRequest(BaseMessage):
    """force a timesync"""
    __meta__ = MessageMeta(device_type=0, id=1, min_length=0, max_length=8)



__all__ = ['MessageType', 'EnumerateRequest', 'TimesyncRequest']

type MessageType = EnumerateRequest | TimesyncRequest