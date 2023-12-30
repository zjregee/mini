#include "proposer.h"

namespace paxos {

Proposer::Proposer(size_t proposer_count, size_t acceptor_count) {
    proposer_count_ = proposer_count;
    acceptor_count_ = acceptor_count;
}

void Proposer::start_propose(Proposal &value) {
    value_ = value;
	is_propose_finished_ = false;
	is_accept_finished_ = false;
	ok_count_ = 0;
	refuse_count_ = 0;
    max_serial_num_ = 0;
}

bool Proposer::start_accept() {
    return is_propose_finished_;
}

bool Proposer::proposed(bool ok, Proposal &last_accept_value) {
	if (is_propose_finished_) {
        return true;
    }
	if (!ok)  {
		refuse_count_++;
        if (refuse_count_ > acceptor_count_ / 2) {
			return false;
        }
        return true;
	}
	ok_count_++;
    if (last_accept_value.serial_num > max_serial_num_) {
		max_serial_num_ = last_accept_value.serial_num;
		value_.value = last_accept_value.value;
	}
	if (ok_count_ > acceptor_count_ / 2) {
		ok_count_ = 0;
        refuse_count_ = 0;
		is_propose_finished_ = true;
	}
	return true;
}

bool Proposer::accepted(bool ok) {
	if (!is_propose_finished_) {
        return true;
    }
	if (!ok) {
		refuse_count_++;
        if (refuse_count_ > acceptor_count_ / 2) {
			return false;
        }
		return true;
	}
	ok_count_++;
	if (ok_count_ > acceptor_count_ / 2) {
        is_accept_finished_ = true;
    }
	return true;
}

Proposal &Proposer::get_proposal() {
    return value_;
}

bool Proposer::is_finished() {
    return is_accept_finished_;
}
}