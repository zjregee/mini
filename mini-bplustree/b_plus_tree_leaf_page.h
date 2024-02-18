#pragma once

#include "b_plus_tree_page.h"

namespace minibplustree {

#define LEAF_PAGE_HEADER_SIZE PAGE_HEADER_SIZE + sizeof(size_t)

#define LEAF_PAGE_SIZE ((4096 - LEAF_PAGE_HEADER_SIZE) / sizeof(MappingType))

class BPlusTreeLeafPage : public BPlusTreePage {
public:
    void Init(size_t page_id, size_t parent_id = INVALID_PAGE_ID, int max_size = LEAF_PAGE_SIZE);
    auto GetNextPageId() const -> size_t;
    void SetNextPageId(size_t next_page_id);
    auto KeyAt(int index) const -> KeyType;
    void SetKeyAt(int index, const KeyType &key);
    auto ValueAt(int index) const -> ValueType;
    void SetValueAt(int index, const ValueType &value);

private:
    size_t next_page_id_;
    MappingType array_[1];
};

}