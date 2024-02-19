#include "page.h"

namespace minibplustree {

Page::Page() {
    data_ = new char[4096];
}

Page::~Page() {
    delete[] data_;
}

auto Page::GetData() -> char* {
    return data_;
}

void Page::ResetMemory() {
    memset(data_, 0, 4096);
}

}