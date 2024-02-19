#pragma once

#include <string>
#include <fcntl.h>
#include <unistd.h>
#include <iostream>

#include "page.h"

namespace minibplustree {

class DiskManager {
public:
    explicit DiskManager(std::string disk_name);
    ~DiskManager();
    auto GetNextPageId() const -> int;
    auto FetchPage(size_t page_id) -> Page *;
    void UnpinPage(size_t page_id, Page *page, bool is_dirty);
    auto NewPage(size_t *page_id) -> Page *;

private:
    int disk_fd_;
    int next_page_id_;
};

}