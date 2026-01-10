#include "redux/canand/MessageBus.h"
#include "redux/canand/CanandEventLoop.h"
#include "ReduxCore.h"
#include "ReduxFIFO.h"
#include <stdexcept>
#include <fmt/format.h>

namespace redux::canand {
    MessageBus MessageBus::ByBusString(std::string busString) {
        uint16_t bus = 0;
        redux::canand::EnsureCANLinkServer();
        
        ReduxFIFO_Status status = ReduxFIFO_OpenBus(busString.c_str(), &bus);
        if (status != 0) {
            const char* msg = ReduxFIFO_ErrorMessage(status);
            throw std::runtime_error(fmt::format("Failed to open bus `{}`: {}", busString, msg));
        } else {
            if (ReduxCore_OpenBusById(bus) < 0) {
                throw std::runtime_error(fmt::format("Failed to open bus `{}`: event loop not initialized", busString));
            }

            return MessageBus{bus};
        }
    }
}