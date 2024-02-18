#include "disk_manager.h"

namespace minibplustree {

DiskManager::DiskManager() {
    next_page_id_ = 0;
    disk_name_ = "/dev/nvme1n1";
    disk_fd_ = open(disk_name_.c_str(), O_RDWR);
    if (disk_fd_ == -1) {
        exit(1);
    }
}

DiskManager::~DiskManager() {
    if (close(disk_fd_) == -1) {
        exit(1);
    }
}

auto DiskManager::FetchPage(size_t page_id) -> Page * {
    Page *page = new Page(page_id);
    page->ResetMemory();
    off_t off = page_id * 4096;
    if (lseek(disk_fd_, off, SEEK_SET) == -1) {
        exit(1);
    }
    if (read(disk_fd_, page->GetData(), 4096) == -1) {
        exit(1);
    }
    return page;
}

void DiskManager::UnpinPage(size_t page_id, Page *page, bool is_dirty) {
    if (is_dirty) {
        off_t off = page_id * 4096;
        if (lseek(disk_fd_, off, SEEK_SET) == -1) {
            exit(1);
        }
        if (write(disk_fd_, page->GetData(), 4096) == -1) {
            exit(1);
        }
    }
    delete page;
}

auto DiskManager::NewPage(size_t *page_id) -> Page * {
    *page_id = next_page_id_;
    next_page_id_++;
    Page *page = new Page(*page_id);
    page->ResetMemory();
    off_t off = *page_id * 4096;
    if (lseek(disk_fd_, off, SEEK_SET) == -1) {
        exit(1);
    }
    if (write(disk_fd_, page->GetData(), 4096) == -1) {
        exit(1);
    }
    return page;
}

}