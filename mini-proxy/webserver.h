#ifndef WEBSERVER_H
#define WEBSERVER_H

#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <stdio.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <cassert>
#include <sys/epoll.h>

#include "threadpool.h"
#include "./http/http_conn.h"

const int MAX_FD = 65536;           // 最大文件描述符
const int MAX_EVENT_NUMBER = 10000; // 最大事件数
const int TIMESLOT = 5;             // 最小超时单位

class WebServer {
public:
    WebServer();
    ~WebServer();
    void init(int port, int opt_linger, int trigmode, int thread_num, int test);
    void thread_pool();
    void sql_pool();
    void log_write();
    void trig_mode();
    void eventListen();
    void eventLoop();
    void timer(int connfd, struct sockaddr_in client_address);
    void adjust_timer(util_timer *timer);
    void deal_time(util_timer *timer, int sockfd);
    bool deal_client();
    bool deal_signal(bool& timeout, bool& stop_server);
    void deal_read(int sockfd);
    void deal_write(int sockfd);

public:
    int m_port;
    int m_test;
    int m_pipefd[2];
    int m_epollfd;
    http_conn *users;
    threadpool<http_conn> *m_pool;
    int m_thread_num;
    epoll_event events[MAX_EVENT_NUMBER];
    int m_listenfd;
    int m_OPT_LINGER;
    int m_TRIGMode;
    int m_LISTENTrigmode;
    int m_CONNTrigmode;
    client_data *users_timer;
    Utils utils;
};

#endif