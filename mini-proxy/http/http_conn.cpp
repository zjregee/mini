#include "http_conn.h"

const char *ok_200_title = "OK";

int setnonblocking(int fd) {
    int old_option = fcntl(fd, F_GETFL);
    int new_option = old_option | O_NONBLOCK;
    fcntl(fd, F_SETFL, new_option);
    return old_option;
}

void addfd(int epollfd, int fd, bool one_shot, int TRIGMode) {
    epoll_event event;
    event.data.fd = fd;

    if (1 == TRIGMode)
        event.events = EPOLLIN | EPOLLET | EPOLLRDHUP;
    else
        event.events = EPOLLIN | EPOLLRDHUP;

    if (one_shot)
        event.events |= EPOLLONESHOT;
    epoll_ctl(epollfd, EPOLL_CTL_ADD, fd, &event);
    setnonblocking(fd);
}

void removefd(int epollfd, int fd) {
    epoll_ctl(epollfd, EPOLL_CTL_DEL, fd, 0);
    close(fd);
}

void modfd(int epollfd, int fd, int ev, int TRIGMode) {
    epoll_event event;
    event.data.fd = fd;

    if (1 == TRIGMode)
        event.events = ev | EPOLLET | EPOLLONESHOT | EPOLLRDHUP;
    else
        event.events = ev | EPOLLONESHOT | EPOLLRDHUP;

    epoll_ctl(epollfd, EPOLL_CTL_MOD, fd, &event);
}

int http_conn::m_user_count = 0;
int http_conn::m_epollfd = -1;
int http_conn::m_test = 0;
int http_conn::server_num = 1;
char *http_conn::server_ip[10] = {"127.0.0.1"};
int http_conn::server_port[10] = {8333};


void http_conn::close_conn(bool real_close) {
    if (real_close && (m_sockfd != -1)) {
        printf("close %d\n", m_sockfd);
        removefd(m_epollfd, m_sockfd);
        m_sockfd = -1;
        m_user_count--;
    }
}

void http_conn::init(int sockfd, int TRIGMode) {
    m_sockfd = sockfd;
    addfd(m_epollfd, sockfd, true, m_TRIGMode);
    m_user_count++;
    m_TRIGMode = TRIGMode;
    init();
}

void http_conn::init() {
    bytes_to_send = 0;
    bytes_have_send = 0;
    m_linger = false;
    m_read_idx = 0;
    m_write_idx = 0;
    timer_flag = 0;
    improv = 0;

    memset(m_read_buf, '\0', READ_BUFFER_SIZE);
    memset(m_write_buf, '\0', WRITE_BUFFER_SIZE);
}

http_conn::HTTP_CODE http_conn::do_request() {
    if (m_test) {
        return FILE_REQUEST;
    }
    int sockfd, num;
    char buf[8192];
    struct hostent *he;
    struct sockaddr_in server;

    srand((unsigned)time(NULL));
    int n = (rand() % server_num);
 
    if ((he = gethostbyname(server_ip[n])) == NULL) {
        printf("gethostbyname() error\n");
        exit(1);
    }
 
    if((sockfd = socket(AF_INET,SOCK_STREAM, 0)) == -1) {
        printf("socket() error\n");
        exit(1);
    }

    bzero(&server,sizeof(server));
    server.sin_family = AF_INET;
    server.sin_port = htons(server_port[n]);
    server.sin_addr = *((struct in_addr *)he->h_addr);

    if(connect(sockfd, (struct sockaddr *)&server, sizeof(server)) == -1) {
        printf("connect() error\n");
        exit(1);
    }

    if((num = send(sockfd, m_read_buf, sizeof(m_read_buf), 1)) == -1){
        printf("send() error\n");
        exit(1);
    }

    if((num = recv(sockfd, buf, 8192, 0)) == -1) {
        printf("recv() error\n");
        exit(1);
    }
    buf[num]='\0';

    close(sockfd);

    for (int i = 0; i <= num; i++) {
        m_write_buf[i] = buf[i];
        m_write_idx++;
    }

    return FILE_REQUEST;
}

bool http_conn::read_once() {
    if (m_read_idx >= READ_BUFFER_SIZE) {
        return false;
    }
    int bytes_read = 0;
    if (0 == m_TRIGMode) {
        //LT读取数据
        bytes_read = recv(m_sockfd, m_read_buf + m_read_idx, READ_BUFFER_SIZE - m_read_idx, 0);
        m_read_idx += bytes_read;
        if (bytes_read <= 0) {
            return false;
        }
        return true;
    } else {
        //ET读数据
        while (true) {
            bytes_read = recv(m_sockfd, m_read_buf + m_read_idx, READ_BUFFER_SIZE - m_read_idx, 0);
            if (bytes_read == -1) {
                if (errno == EAGAIN || errno == EWOULDBLOCK)
                    break;
                return false;
            } else if (bytes_read == 0) {
                return false;
            }
            m_read_idx += bytes_read;
        }
        return true;
    }
}

bool http_conn::write() {
    int temp = 0;
    if (bytes_to_send == 0) {
        modfd(m_epollfd, m_sockfd, EPOLLIN, m_TRIGMode);
        init();
        return true;
    }
    while (1) {
        temp = writev(m_sockfd, m_iv, m_iv_count);
        if (temp < 0) {
            if (errno == EAGAIN) {
                modfd(m_epollfd, m_sockfd, EPOLLOUT, m_TRIGMode);
                return true;
            }
            return false;
        }
        bytes_have_send += temp;
        bytes_to_send -= temp;
        m_iv[0].iov_base = m_write_buf + bytes_have_send;
        m_iv[0].iov_len = m_iv[0].iov_len - bytes_have_send;
        if (bytes_to_send <= 0) {
            modfd(m_epollfd, m_sockfd, EPOLLIN, m_TRIGMode);
            if (m_linger) {
                init();
                return true;
            } else {
                return false;
            }
        }
    }
}

http_conn::HTTP_CODE http_conn::process_read() {
    return do_request();
}

bool http_conn::process_write(HTTP_CODE ret) {
    if (m_test) {
        add_status_line(200, ok_200_title);
        const char *ok_string = "<html><body></body></html>";
        add_headers(strlen(ok_string));
        if (!add_content(ok_string))
            return false;
    }
    m_iv[0].iov_base = m_write_buf;
    m_iv[0].iov_len = m_write_idx;
    m_iv_count = 1;
    bytes_to_send = m_write_idx;
    return true;
}

void http_conn::process() {
    HTTP_CODE read_ret = process_read();
    if (read_ret == NO_REQUEST) {
        modfd(m_epollfd, m_sockfd, EPOLLIN, m_TRIGMode);
        return;
    }
    bool write_ret = process_write(read_ret);
    if (!write_ret) {
        close_conn();
    }
    modfd(m_epollfd, m_sockfd, EPOLLOUT, m_TRIGMode);
}

bool http_conn::add_response(const char *format, ...)
{
    if (m_write_idx >= WRITE_BUFFER_SIZE)
        return false;
    va_list arg_list;
    va_start(arg_list, format);
    int len = vsnprintf(m_write_buf + m_write_idx, WRITE_BUFFER_SIZE - 1 - m_write_idx, format, arg_list);
    if (len >= (WRITE_BUFFER_SIZE - 1 - m_write_idx)) {
        va_end(arg_list);
        return false;
    }
    m_write_idx += len;
    va_end(arg_list);
    return true;
}

bool http_conn::add_status_line(int status, const char *title) {
    return add_response("%s %d %s\r\n", "HTTP/1.1", status, title);
}

bool http_conn::add_headers(int content_len) {
    return add_content_length(content_len) && add_linger() &&
           add_blank_line();
}

bool http_conn::add_content_length(int content_len) {
    return add_response("Content-Length:%d\r\n", content_len);
}

bool http_conn::add_content_type() {
    return add_response("Content-Type:%s\r\n", "text/html");
}

bool http_conn::add_linger() {
    return add_response("Connection:%s\r\n", (m_linger == true) ? "keep-alive" : "close");
}

bool http_conn::add_blank_line() {
    return add_response("%s", "\r\n");
}

bool http_conn::add_content(const char *content) {
    return add_response("%s", content);
}