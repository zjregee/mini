#include <mutex>
#include <thread>
#include <iostream>
#include "paxos_data.h"
#include "acceptor.h"
#include "proposer.h"

const int proposer_count = 10;
const int acceptor_count = 21;

std::mutex m[acceptor_count];
paxos::Acceptor a[acceptor_count];
std::mutex g_m;
size_t finish_count = 0;

void propose_loop(size_t index) {
    paxos::Proposer proposer(proposer_count, acceptor_count);
    paxos::Proposal proposal;
    proposal.serial_num = index + 1;
    proposal.value = index + 1;
    while (true) {
        size_t id[acceptor_count];
        size_t count = 0;
        for (size_t i = 0; i < acceptor_count; i++) {
            std::this_thread::sleep_for(std::chrono::milliseconds(std::rand() % 100));
            std::unique_lock<std::mutex> lock(m[i]);
            bool ok = a[i].propose(proposal.serial_num);
            lock.unlock();
            std::this_thread::sleep_for(std::chrono::milliseconds(std::rand() % 100));
            if (!proposer.proposed(ok)) {
                break;
            }
            id[count++] = i;
            if (proposer.is_propose_finished()) {
                break;
            }
        }
        if (!proposer.is_propose_finished()) {
            proposal.serial_num += proposer_count;
            proposer.restart();
            continue;
        }
        for (size_t i = 0; i < count; i++) {
            std::this_thread::sleep_for(std::chrono::milliseconds(std::rand() % 100));
            std::unique_lock<std::mutex> lock(m[id[i]]);
            bool ok = a[id[i]].accept(proposal);
            lock.unlock();
            std::this_thread::sleep_for(std::chrono::milliseconds(std::rand() % 100));
            if (!proposer.accepted(ok)) {
                break;
            }
            if (proposer.is_accept_finished()) {
                break;
            }
        }
        if (!proposer.is_accept_finished()) {
            proposal.serial_num += proposer_count;
            proposer.restart();
            continue;
        }
        std::lock_guard<std::mutex> lock(g_m);
        std::cout << "proposer " << index << " accepted, serial_num: " << proposal.serial_num << " value: " << proposal.value << "." << std::endl;
        finish_count++;
        break;
    }
}

int main() {
    for (size_t i = 0; i < proposer_count; i++) {
        std::thread t(propose_loop, i);
        t.detach();
    }
    while (true) {
        if (finish_count == proposer_count) {
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }
    std::cout << "paxos simulation finished." << std::endl;
    return 0;
}