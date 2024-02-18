#pragma once

#include "b_plus_tree_page.h"

namespace minibplustree {

#define INTERNAL_PAGE_HEADER_SIZE PAGE_HEADER_SIZE

#define INTERNAL_PAGE_SIZE ((4096 - INTERNAL_PAGE_HEADER_SIZE) / (sizeof(MappingType)))

class BPlusTreeInternalPage : public BPlusTreePage {
public:
    void Init(size_t page_id, size_t parent_id = INVALID_PAGE_ID, int max_size = INTERNAL_PAGE_SIZE);
    auto KeyAt(int index) const -> KeyType;
    void SetKeyAt(int index, const KeyType &key);
    auto ValueAt(int index) const -> ValueType;
    void SetValueAt(int index, const ValueType &value);

private:
    MappingType array_[1];
};

}