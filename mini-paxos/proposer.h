#pragma once

#include "paxos_data.h"

namespace paxos {

class Proposer {
public:
    Proposer(size_t proposer_count, size_t acceptor_count);
    void restart();
    bool proposed(bool ok);
    bool accepted(bool ok);
    bool is_propose_finished();
    bool is_accept_finished();

private:
    size_t proposer_count_;
    size_t acceptor_count_;
    bool is_propose_finished_;
    bool is_accept_finished_;
    size_t ok_count_;
    size_t refuse_count_;
};

}