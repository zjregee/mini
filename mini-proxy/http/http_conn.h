#ifndef HTTPCONNECTION_H
#define HTTPCONNECTION_H
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/epoll.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <assert.h>
#include <sys/stat.h>
#include <string.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <stdarg.h>
#include <errno.h>
#include <sys/wait.h>
#include <sys/uio.h>
#include <map>
#include <netdb.h>

#include "../timer/lst_timer.h"

class http_conn {
public:
    static const int READ_BUFFER_SIZE = 2048;
    static const int WRITE_BUFFER_SIZE = 8192;
    enum HTTP_CODE {
        NO_REQUEST,
        GET_REQUEST,
        BAD_REQUEST,
        NO_RESOURCE,
        FORBIDDEN_REQUEST,
        FILE_REQUEST,
        INTERNAL_ERROR,
        CLOSED_CONNECTION
    };
public:
    http_conn() {}
    ~http_conn() {}
public:
    void init(int sockfd, int);
    void close_conn(bool real_close = true);
    void process();
    bool read_once();
    bool write();
    int timer_flag;
    int improv;
private:
    void init();
    HTTP_CODE process_read();
    bool process_write(HTTP_CODE ret);
    HTTP_CODE do_request();
    bool add_response(const char *format, ...);
    bool add_content(const char *content);
    bool add_status_line(int status, const char *title);
    bool add_headers(int content_length);
    bool add_content_type();
    bool add_content_length(int content_length);
    bool add_linger();
    bool add_blank_line();
public:
    static int m_epollfd;
    static int m_user_count;
    static int m_test;
    static int server_num;
    static char *server_ip[10];
    static int server_port[10];
    int m_state;
private:
    int m_sockfd;
    char m_read_buf[READ_BUFFER_SIZE];
    int m_read_idx;
    char m_write_buf[WRITE_BUFFER_SIZE];
    int m_write_idx;
    bool m_linger;
    struct iovec m_iv[2];
    int m_iv_count;
    int bytes_to_send;
    int bytes_have_send;
    int m_TRIGMode;
};

#endif