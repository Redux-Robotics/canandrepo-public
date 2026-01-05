
from pycanandmessage import BaseDevice, MessageWrapper
from . import types, message as msg, setting as stg

class Reduxbroadcast(BaseDevice):
    device_type = 0
    msg = msg
    stg = stg
    types = types

    name = 'Reduxbroadcast'
    messages = {
        0: msg.EnumerateRequest,
        1: msg.TimesyncRequest,
    }

    @classmethod
    def decode_msg(cls, msg: MessageWrapper) -> msg.MessageType | None:
        return cls.decode_msg_generic(msg)
