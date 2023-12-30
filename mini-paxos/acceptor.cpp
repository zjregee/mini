#include "acceptor.h"

namespace paxos {

Acceptor::Acceptor() {
    max_serial_num_ = 0;
}

bool Acceptor::propose(size_t serial_num) {
    if (serial_num == 0) {
        return false;
    }
    if (max_serial_num_ > serial_num) {
        return false;
    }
    return true;
}

bool Acceptor::accept(Proposal &value) {
    if (value.serial_num == 0) {
        return false;
    }
    if (max_serial_num_ > value.serial_num) {
        return false;
    }
    max_serial_num_ = value.serial_num;
    return true;
}

}