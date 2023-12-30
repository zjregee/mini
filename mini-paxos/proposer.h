#pragma once

#include "paxos_data.h"

namespace paxos {

class Proposer {
public:
    Proposer(size_t proposer_count, size_t acceptor_count);
    void start_propose(Proposal &value);
    bool start_accept();
    bool proposed(bool ok, Proposal &last_accept_value);
    bool accepted(bool ok);
    Proposal &get_proposal();
    bool is_finished();

private:
    size_t proposer_count_;
    size_t acceptor_count_;
    bool is_propose_finished_;
    bool is_accept_finished_;
    size_t ok_count_;
    size_t refuse_count_;
    size_t max_serial_num_;
    Proposal value_;
};

}