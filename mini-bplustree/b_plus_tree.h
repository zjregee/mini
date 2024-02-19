#pragma once

#include "page.h"
#include "disk_manager.h"
#include "index_iterator.h"
#include "b_plus_tree_page.h"
#include "b_plus_tree_leaf_page.h"
#include "b_plus_tree_internal_page.h"

namespace minibplustree {

class BPlusTree {
public:
    explicit BPlusTree(DiskManager *disk_manager);
    auto IsEmpty() const -> bool;
    auto GetRootPageId() const -> int;
    auto Insert(const KeyType &key, const ValueType &value) -> bool;
    void Remove(const KeyType &key);
    auto GetValue(const KeyType &key, ValueType *result) -> bool;
    auto Begin() -> IndexIterator;
    auto Begin(const KeyType &key) -> IndexIterator;

    void PrintInternal(size_t page_id) {
        auto *raw_cursor_page = disk_manager_->FetchPage(page_id);
        auto *cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
        if (cursor_page->IsLeafPage()) {
            return;
        }
        auto *internal_page = static_cast<BPlusTreeInternalPage *>(cursor_page);
        std::cout << internal_page->Debug();
        for (size_t i = 0; i < internal_page->GetSize(); i++) {
            PrintInternal(internal_page->ValueAt(i));
        }
        disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
    }

private:
    int root_page_id_;
    DiskManager *disk_manager_;
    KeyComparator comparator_;

    void InsertInternal(KeyType key, size_t left_page_id, size_t right_page_id, size_t parent_page_id);
};

}