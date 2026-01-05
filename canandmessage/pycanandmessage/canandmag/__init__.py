
from pycanandmessage import BaseDevice, MessageWrapper
from . import types, message as msg, setting as stg

class Canandmag(BaseDevice):
    device_type = 7
    msg = msg
    stg = stg
    types = types

    name = 'Canandmag'
    messages = {
        0: msg.CanIdArbitrate,
        1: msg.CanIdError,
        2: msg.SettingCommand,
        3: msg.SetSetting,
        4: msg.ReportSetting,
        5: msg.ClearStickyFaults,
        6: msg.Status,
        7: msg.PartyMode,
        8: msg.OtaData,
        9: msg.OtaToHost,
        10: msg.OtaToDevice,
        11: msg.Enumerate,
        12: msg.AtomicBondAnnouncement,
        13: msg.AtomicBondSpecification,
        31: msg.PositionOutput,
        30: msg.VelocityOutput,
        29: msg.RawPositionOutput,
    }

    @classmethod
    def decode_msg(cls, msg: MessageWrapper) -> msg.MessageType | None:
        return cls.decode_msg_generic(msg)
