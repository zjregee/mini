#include "acceptor.h"

namespace paxos {

Acceptor::Acceptor() {
    max_serial_num_ = 0;
    last_accept_value_.serial_num = 0;
    last_accept_value_.value = 0;
}

bool Acceptor::propose(size_t serial_num, Proposal &last_accept_value) {
    if (serial_num == 0) {
        return false;
    }
    if (max_serial_num_ > serial_num) {
        return false;
    }
    max_serial_num_ = serial_num;
    last_accept_value = last_accept_value_;
    return true;
}

bool Acceptor::accept(Proposal &value) {
    if (value.serial_num == 0) {
        return false;
    }
    if (max_serial_num_ > value.serial_num) {
        return false;
    }
    last_accept_value_ = value;
    return true;
}

}