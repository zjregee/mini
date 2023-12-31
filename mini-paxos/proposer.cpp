#include "proposer.h"

namespace paxos {

Proposer::Proposer(size_t proposer_count, size_t acceptor_count) {
    proposer_count_ = proposer_count;
    acceptor_count_ = acceptor_count;
}

void Proposer::restart() {
	is_propose_finished_ = false;
	is_accept_finished_ = false;
	ok_count_ = 0;
	refuse_count_ = 0;
}

bool Proposer::proposed(bool ok) {
	if (!ok)  {
		refuse_count_++;
        if (refuse_count_ > acceptor_count_ / 2) {
			return false;
        }
        return true;
	}
	ok_count_++;
	if (ok_count_ > acceptor_count_ / 2) {
		ok_count_ = 0;
        refuse_count_ = 0;
		is_propose_finished_ = true;
	}
	return true;
}

bool Proposer::accepted(bool ok) {
	if (!ok) {
		refuse_count_++;
        if (refuse_count_ > acceptor_count_ / 2) {
			return false;
        }
		return true;
	}
	ok_count_++;
	if (ok_count_ > acceptor_count_ / 2) {
		ok_count_ = 0;
		refuse_count_ = 0;
        is_accept_finished_ = true;
    }
	return true;
}

bool Proposer::is_propose_finished() {
    return is_propose_finished_;
}

bool Proposer::is_accept_finished() {
    return is_accept_finished_;
}
}