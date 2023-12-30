#pragma once

#include "paxos_data.h"

namespace paxos {

class Acceptor {
public:
    Acceptor();
    bool propose(size_t serial_num);
    bool accept(Proposal &value);

private:
    size_t max_serial_num_;
};

}