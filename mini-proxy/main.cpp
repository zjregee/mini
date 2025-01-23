#include "webserver.h"

int main(int argc, char *argv[]) {
    WebServer server;
    server.init(8080, 0, 0, 8, 0);
    server.thread_pool();
    server.trig_mode();
    server.eventListen();
    server.eventLoop();
    
    return 0;
}