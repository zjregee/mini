#include <mutex>
#include <thread>
#include "paxos_data.h"
#include "acceptor.h"
#include "proposer.h"

std::mutex m[21];
paxos::Proposer p[10];
paxos::Acceptor a[21];
size_t finish_count = 0;



int main() {
    for (int i = 0; i < 10; i++) {

    }
    for (int i = 0; i < 10; i++) {

    }
    while (true) {
        if (finish_count < 10) {
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    return 0;
}