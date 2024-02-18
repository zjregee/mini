#pragma once

#include <string>
#include <fcntl.h>
#include <unistd.h>

#include "page.h"

namespace minibplustree {

class DiskManager {
public:
    DiskManager();
    ~DiskManager();
    auto FetchPage(size_t page_id) -> Page *;
    void UnpinPage(size_t page_id, Page *page, bool is_dirty);
    auto NewPage(size_t *page_id) -> Page *;

private:
    int disk_fd_;
    int next_page_id_;
    std::string disk_name_;
};

}