import enum
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *

class AtomicBondBusRate(enum.IntEnum):
    RATE_1M_2B = 0x0
    """1 megabit/s CAN 2.0B"""
    RATE_RESERVED_0 = 0x1
    """1 megabit/s CAN-FD"""
    RATE_RESERVED_1 = 0x2
    """5 megabit/s CAN-FD"""
    RATE_RESERVED_2 = 0x3
    """8 megabit/s CAN-FD"""

class Setting(enum.IntEnum):


class SettingCommand(enum.IntEnum):




