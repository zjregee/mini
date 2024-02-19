#pragma once

#include "page.h"
#include "disk_manager.h"
#include "b_plus_tree_page.h"
#include "b_plus_tree_leaf_page.h"

namespace minibplustree {

class IndexIterator {
public:
    IndexIterator(DiskManager *disk_manager, size_t start_page_id, size_t start_index);
    auto IsEnd() -> bool;
    auto operator*() -> const MappingType &;
    auto operator++() -> IndexIterator &;
    auto operator==(const IndexIterator &itr) const -> bool;
    auto operator!=(const IndexIterator &itr) const -> bool;

private:
    size_t current_page_id_;
    size_t current_index_;
    MappingType current_data_;
    DiskManager *disk_manager_;
};

}