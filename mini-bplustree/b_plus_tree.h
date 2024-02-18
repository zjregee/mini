#pragma once

#include "page.h"
#include "disk_manager.h"
#include "b_plus_tree_page.h"
#include "b_plus_tree_leaf_page.h"
#include "b_plus_tree_internal_page.h"

namespace minibplustree {

class BPlusTree {
public:
    explicit BPlusTree(DiskManager *disk_manager);
    auto IsEmpty() const -> bool;
    auto Insert(const KeyType &key, const ValueType &value) -> bool;
    void Remove(const KeyType &key);
    auto GetValue(const KeyType &key, ValueType *result) -> bool;

private:
    int root_page_id_;
    DiskManager *disk_manager_;
    KeyComparator comparator_;

    void InsertInternal(KeyType key, size_t left_page_id, size_t right_page_id, size_t parent_page_id);
};

}