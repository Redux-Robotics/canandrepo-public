import can
import time
from pycanandmessage import BaseDevice, BaseMessage, BaseSetting, MessageWrapper, cananddevice
from typing import Type, Optional, List, Tuple, Set, Iterable

REDUX_CAN_VENDOR_ID = 0xE

# Mask for matching can id + everything that's not apiIndex
CAN_MESSAGE_MASK = 0x1fff003f

class CANDevice:
    def __init__(self, bus: can.Bus, device: Type[BaseDevice], dev_id: int, recv_timeout=2):
        self.bus: can.BusABC = bus
        self.device: Type[BaseDevice] = device
        self.dev_id: int = dev_id
        self.recv_timeout: int = recv_timeout

        self.base_id: int = self.addr()
    
    def addr(self, api_index: int = 0) -> int:
        """Construct a full 29-bit can address with the given api index"""
        return ((self.device.device_type & 0x1F) << 24) | ((REDUX_CAN_VENDOR_ID & 0xFF) << 16) | ((api_index & 0xFF) << 6) | (self.dev_id & 0x3F)
    
    def send_enumerate(self):
        self.bus.send(can.Message(arbitration_id=0xE0000, data = [], is_extended_id=True))
    
    def send_wrapper(self, msg: MessageWrapper, timeout=2) -> can.Message:
        """Send a message on bus. Returns the can.Message sent."""
        self.bus.send(msg.to_can(), timeout=timeout)
        return msg
    
    def recv_wrapper(self, msg: Type[BaseMessage], timeout=2) -> Optional[MessageWrapper]:
        """Receives a message by id."""
        m = self._get_msg_by_id(self.addr(msg.__meta__.id), timeout=timeout)
        if m is None:
            return None
        return MessageWrapper.from_can(m)

    
    def send_msg(self, msg: BaseMessage, timeout=2) -> can.Message:
        """this overrides device type with its own."""
        return self.send_wrapper(msg.to_wrapper(self.dev_id, device_type=self.device.device_type), timeout=timeout)
    
    def recv_msg[T: BaseMessage](self, msg: Type[T], timeout=2, msg_filter=lambda x: True) -> Optional[T]:
        """this is able to receive generic messages as well"""
        start_time = time.monotonic()
        while (time.monotonic() - start_time) < timeout:
            can_msg = self.recv_wrapper(msg, timeout=timeout)
            if can_msg is None:
                return None
            data = msg.from_wrapper(can_msg)
            if data is not None and msg_filter(data):
                return data
        return None


    def fetch_setting[T: BaseSetting](self, setting: Type[T], timeout=2) -> Optional[T]:
        """Fetch setting."""

        setting_id = setting.__meta__.idx
        #self.drain()
        msg = cananddevice.msg.SettingCommand(cananddevice.types.SettingCommand.FETCH_SETTING_VALUE, setting_id)


        self.send_msg(cananddevice.msg.SettingCommand(cananddevice.types.SettingCommand.FETCH_SETTING_VALUE, setting_id), timeout=timeout)

        msg_filter = lambda x: x.address == setting_id
        msg: Optional[cananddevice.msg.ReportSetting] = self.recv_msg(setting.__meta__.report_setting, timeout=timeout, msg_filter=msg_filter)
        if msg is None:
            return None
        return setting.decode(msg.value)

    
    def set_setting[T: BaseSetting](self, setting: T, timeout=2, flags=None, check=True) -> Optional[T]:
        """Set setting."""
        if flags is None:
            flags = cananddevice.types.SettingFlags(False, False, 0)
        self.send_msg(cananddevice.msg.SetSetting(setting.__meta__.idx, setting.encode(), flags), timeout=timeout)
        if not check:
            return None

        def msg_filter(x: cananddevice.msg.ReportSetting) -> bool:
            return x.address == setting.__meta__.idx

        msg: Optional[cananddevice.msg.ReportSetting] = self.recv_msg(setting.__meta__.report_setting, timeout=timeout, msg_filter=msg_filter)
        if msg is None:
            return None
        return setting.decode(msg.value)

    def drain(self) -> int:
        """Drains internal can bus buffers by reading at 0 timeout until no messages can be received anymore"""
        cnt = 0
        while self.bus.recv(0) is not None:
            time.sleep(0.00001)
            cnt += 1
            continue
        return cnt
    
    def collect(self, time_sec: float, msg_id=0, mask=CAN_MESSAGE_MASK, drain=True) -> List[can.Message]:
        """Collects messages over a time frame that maches the id
        Messages are not interpreted. 
        """
        if drain:
            self.drain()
        addr_base = self.addr(msg_id) & mask
        start_time = time.monotonic()
        ret = []
        while (time.monotonic() - start_time) < time_sec:
            msg: can.Message = self.bus.recv(timeout=time_sec)
            if msg is None:
                continue
            if (msg.arbitration_id & mask) != addr_base:
                continue
            ret.append(msg)
        return ret

    def monitor(self, time_sec: float, drain=True) -> List[BaseMessage]:
        if drain:
            self.drain()
        start_time = time.monotonic()
        ret = []
        while (time.monotonic() - start_time) < time_sec:
            msg: can.Message | None = self.bus.recv(timeout=time_sec)
            if msg is None:
                continue
            decoded = self.device.decode_msg_generic(MessageWrapper.from_can(msg))
            if decoded is not None:
                ret.append(decoded)
        return ret


    def parse_msgs(self, devclasses: Iterable[BaseDevice], msgs: List[can.Message]) -> List[Tuple[can.Message, Optional[BaseMessage]]]:
        parsed: List[BaseMessage] = []
        for msg in msgs:
            result = None
            for cls in devclasses:
                decoded = cls.decode_msg_generic(MessageWrapper.from_can(msg))
                if decoded is not None:
                    result = decoded
                    break
            parsed.append((msg, result))
        return parsed


    def _get_msg_by_id(self, msg_id: int, timeout: float) -> Optional[can.Message]:
        start_time = time.monotonic()
        while (time.monotonic() - start_time) < timeout:
            resp = self.bus.recv(self.recv_timeout)
            if resp is None:
                continue
            if resp.arbitration_id != msg_id:
                continue
            else:
                return resp
        return None
    
    def count_matching[T: BaseMessage](self, msgs: List[can.Message], msg_type: Type[T]) -> int:
        cnt = 0
        for msg in msgs:
            cnt += int(msg_type.from_wrapper(MessageWrapper.from_can(msg)) is not None)
        return cnt


    def simulate_conflict(self, conflict_msg=None):
        # it doesn't seriously matter what messages this is as long as it's a periodic message
        if conflict_msg is None:
            self.send_msg(cananddevice.msg.Status(b'\x00\x00\x00\x00\x00\x00\x00\x00'))
        else:
            self.send_msg(conflict_msg)

        # collect can arb messages over 3 seconds
        messages = self.collect(3)
        arb_codes: Set[int] = set()
        for rmsg in messages:
            msg: Optional[cananddevice.msg.CanIdError] = self.device.messages[1].from_wrapper(MessageWrapper.from_can(rmsg))
            assert not (msg is None and arb_codes), f"Non-conflict message detected id={rmsg.arbitration_id >> 6 & 0x1f}"
            if msg is None:
                continue
            arb_codes.add(msg.addr_value)

        assert len(arb_codes) > 0, "No can id conflict messages collected (is the device in conflict mode?)"
        assert len(arb_codes) == 1, "Multiple arbitration codes detected. Are there multiple devices in this test setup?"
        return list(arb_codes)[0]

    def simulate_arb(self, arb_id: bytes, same_device=False, wait=0.5):
        # we pretend to arb another device to see if it shuts up.
        # we always use 0xffffffffffffffff because no canandmag will ever have that as a serial unless someone fucked up
        #print("Send message:")
        print(self.send_msg(cananddevice.msg.CanIdArbitrate(arb_id)))
        time.sleep(wait)
        messages = self.collect(3)
        print("Recv message:")
        print(messages)
        assert self.count_matching(messages, self.device.messages[1]) == len(messages), "Device should not be in normal operation!"

    def get_serial(self) -> Optional[bytes]:
        serial: bytes = self.fetch_setting(cananddevice.stg.SerialNumber)
        return serial