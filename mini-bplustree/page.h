#pragma once

#include <cstring>

namespace minibplustree {

class Page {
public:
    explicit Page(size_t page_id);
    ~Page();
    auto GetData() -> char*;
    void ResetMemory();

private:
    size_t page_id_;
    char* data_;
};

}