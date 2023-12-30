#pragma once

#include "paxos_data.h"

namespace paxos {

class Acceptor {
public:
    Acceptor();
    bool propose(size_t serial_num, Proposal &last_accpet_value);
    bool accept(Proposal &value);

private:
    size_t max_serial_num_;
    Proposal last_accpet_value_;
};

}