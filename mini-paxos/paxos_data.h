#pragma once

#include <cstddef>

namespace paxos {

struct Proposal {
    size_t serial_num;
    size_t value;
};

}