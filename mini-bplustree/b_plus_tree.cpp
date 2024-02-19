#include "b_plus_tree.h"
#include <iostream>

namespace minibplustree {

BPlusTree::BPlusTree(DiskManager *disk_manager) : disk_manager_(disk_manager) {
    root_page_id_ = INVALID_PAGE_ID;
    comparator_ = KeyComparator{};
}

auto BPlusTree::IsEmpty() const -> bool {
    return root_page_id_ == INVALID_PAGE_ID;
}

auto BPlusTree::Insert(const KeyType &key, const ValueType &value) -> bool {
    if (IsEmpty()) {
        size_t root_page_id;
        auto *raw_root_page = disk_manager_->NewPage(&root_page_id);
        auto *root_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_root_page->GetData());
        root_page->Init(root_page_id);
        root_page->SetSize(1);
        root_page->SetKeyAt(0, key);
        root_page->SetValueAt(0, value);
        disk_manager_->UnpinPage(root_page_id, raw_root_page, true);
        root_page_id_ = root_page_id;
        return true;
    }
    auto *raw_cursor_page = disk_manager_->FetchPage(root_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
    while (!cursor_page->IsLeafPage()) {
        auto *internal_page = static_cast<BPlusTreeInternalPage *>(cursor_page);
        int size = internal_page->GetSize();
        for (int index = 1; index < size; index++) {
            if (comparator_(key, internal_page->KeyAt(index)) < 0) {
                size_t next_page_id = internal_page->ValueAt(index - 1);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
                break;
            }
            if (index == internal_page->GetSize() - 1) {
                size_t next_page_id = internal_page->ValueAt(index);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
            }
        }
    }
    auto *leaf_page = static_cast<BPlusTreeLeafPage *>(cursor_page);
    int index;
    for (index = 0; index < leaf_page->GetSize(); index++) {
        int cmp = comparator_(key, leaf_page->KeyAt(index));
        if (cmp == 0) {
            disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
            return false;
        }
        if (cmp < 0) {
            break;
        }
    }
    for (int right_index = leaf_page->GetSize(); right_index > index; right_index--) {
        leaf_page->SetKeyAt(right_index, leaf_page->KeyAt(right_index - 1));
        leaf_page->SetValueAt(right_index, leaf_page->ValueAt(right_index - 1));
    }
    leaf_page->SetKeyAt(index, key);
    leaf_page->SetValueAt(index, value);
    leaf_page->IncreaseSize(1);
    if (leaf_page->GetSize() < leaf_page->GetMaxSize()) {
        disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, true);
        return true;
    }
    size_t split_page_id;
    auto *raw_split_page = disk_manager_->NewPage(&split_page_id);
    auto *split_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_split_page->GetData());
    split_page->Init(split_page_id, leaf_page->GetParentPageId());
    split_page->SetNextPageId(leaf_page->GetNextPageId());
    leaf_page->SetNextPageId(split_page_id);
    int leaf_size = leaf_page->GetMaxSize() / 2;
    int split_size = leaf_page->GetMaxSize() - leaf_size;
    leaf_page->SetSize(leaf_size);
    split_page->SetSize(split_size);
    for (int index = 0; index < split_size; index++) {
        split_page->SetKeyAt(index, leaf_page->KeyAt(index + leaf_size));
        split_page->SetValueAt(index, leaf_page->ValueAt(index + leaf_size));
    }
    if (leaf_page->IsRootPage()) {
        size_t root_page_id;
        auto *raw_root_page = disk_manager_->NewPage(&root_page_id);
        auto *root_page = reinterpret_cast<BPlusTreeInternalPage *>(raw_root_page->GetData());
        root_page->Init(root_page_id);
        root_page->SetSize(2);
        root_page->SetValueAt(0, leaf_page->GetPageId());
        root_page->SetKeyAt(1, split_page->KeyAt(0));
        root_page->SetValueAt(1, split_page->GetPageId());
        leaf_page->SetParentPageId(root_page_id);
        split_page->SetParentPageId(root_page_id);
        disk_manager_->UnpinPage(root_page_id, raw_root_page, true);
        disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, true);
        disk_manager_->UnpinPage(split_page->GetPageId(), raw_split_page, true);
        root_page_id_ = root_page_id;
        return true;
    }
    size_t left_page_id = cursor_page->GetPageId();
    size_t right_page_id = split_page->GetPageId();
    size_t parent_page_id = cursor_page->GetParentPageId();
    KeyType insert_key = split_page->KeyAt(0);
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, true);
    disk_manager_->UnpinPage(split_page->GetPageId(), raw_split_page, true);
    InsertInternal(insert_key, left_page_id, right_page_id, parent_page_id);
    return true;
}

void BPlusTree::InsertInternal(KeyType key, size_t left_page_id, size_t right_page_id, size_t parent_page_id) {
    auto *raw_parent_page = disk_manager_->FetchPage(parent_page_id);
    auto *parent_page = reinterpret_cast<BPlusTreeInternalPage *>(raw_parent_page->GetData());
    int index;
    for (index = 1; index < parent_page->GetSize(); index++) {
        if (comparator_(key, parent_page->KeyAt(index)) < 0) {
            break;
        }
    }
    for (int right_index = parent_page->GetSize(); right_index > index; right_index--) {
        parent_page->SetKeyAt(right_index, parent_page->KeyAt(right_index - 1));
        parent_page->SetValueAt(right_index, parent_page->ValueAt(right_index - 1));
    }
    parent_page->SetKeyAt(index, key);
    parent_page->SetValueAt(index, right_page_id);
    parent_page->IncreaseSize(1);
    if (parent_page->GetSize() < parent_page->GetMaxSize()) {
        disk_manager_->UnpinPage(parent_page->GetPageId(), raw_parent_page, true);
        return;
    }
    size_t split_page_id;
    auto *raw_split_page = disk_manager_->NewPage(&split_page_id);
    auto *split_page = reinterpret_cast<BPlusTreeInternalPage *>(raw_split_page->GetData());
    split_page->Init(split_page_id, parent_page->GetParentPageId());
    int parent_size = parent_page->GetMaxSize() / 2;
    int split_size = parent_page->GetMaxSize() - parent_size;
    parent_page->SetSize(parent_size);
    split_page->SetSize(split_size);
    for (int index = 0; index < split_size; index++) {
        if (parent_page->ValueAt(index + parent_size) == left_page_id) {
            auto *raw_page = disk_manager_->FetchPage(left_page_id);
            auto *page = reinterpret_cast<BPlusTreePage *>(raw_page->GetData());
            page->SetParentPageId(split_page_id);
            disk_manager_->UnpinPage(page->GetPageId(), raw_page, true);
        }
        if (parent_page->ValueAt(index + parent_size) == right_page_id) {
            auto *raw_page = disk_manager_->FetchPage(right_page_id);
            auto *page = reinterpret_cast<BPlusTreePage *>(raw_page->GetData());
            page->SetParentPageId(split_page_id);
            disk_manager_->UnpinPage(page->GetPageId(), raw_page, true);
        }
        split_page->SetKeyAt(index, parent_page->KeyAt(index + parent_size));
        split_page->SetValueAt(index, parent_page->ValueAt(index + parent_size));
    }
    if (parent_page->IsRootPage()) {
        size_t root_page_id;
        auto *raw_root_page = disk_manager_->NewPage(&root_page_id);
        auto *root_page = reinterpret_cast<BPlusTreeInternalPage *>(raw_root_page->GetData());
        root_page->Init(root_page_id);
        root_page->SetSize(2);
        root_page->SetValueAt(0, parent_page->GetPageId());
        root_page->SetKeyAt(1, split_page->KeyAt(0));
        root_page->SetValueAt(1, split_page->GetPageId());
        parent_page->SetParentPageId(root_page_id);
        split_page->SetParentPageId(root_page_id);
        disk_manager_->UnpinPage(root_page_id, raw_root_page, true);
        disk_manager_->UnpinPage(parent_page->GetPageId(), raw_parent_page, true);
        disk_manager_->UnpinPage(split_page->GetPageId(), raw_split_page, true);
        root_page_id_ = root_page_id;
        return;
    }
    left_page_id = parent_page->GetPageId();
    right_page_id = split_page->GetPageId();
    parent_page_id = parent_page->GetParentPageId();
    key = split_page->KeyAt(0);
    disk_manager_->UnpinPage(parent_page->GetPageId(), raw_parent_page, true);
    disk_manager_->UnpinPage(split_page->GetPageId(), raw_split_page, true);
    InsertInternal(key, left_page_id, right_page_id, parent_page_id);
}

void BPlusTree::Remove(const KeyType &key) {

}

auto BPlusTree::GetValue(const KeyType &key, ValueType *result) -> bool {
    if (IsEmpty()) {
        return false;
    }
    auto *raw_cursor_page = disk_manager_->FetchPage(root_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
    while (!cursor_page->IsLeafPage()) {
        auto *internal_page = static_cast<BPlusTreeInternalPage *>(cursor_page);
        int size = internal_page->GetSize();
        int page_id = internal_page->GetPageId();
        for (int index = 1; index < size; index++) {
            if (comparator_(key, internal_page->KeyAt(index)) < 0) {
                size_t next_page_id = internal_page->ValueAt(index - 1);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
                break;
            }
            if (index == internal_page->GetSize() - 1) {
                size_t next_page_id = internal_page->ValueAt(index);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
            }
        }
    }
    auto *leaf_page = static_cast<BPlusTreeLeafPage *>(cursor_page);
    for (int index = 0; index < leaf_page->GetSize(); index++) {
        int cmp = comparator_(key, leaf_page->KeyAt(index));
        if (cmp == 0) {
            *result = leaf_page->ValueAt(index);
            disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
            return true;
        }
        if (cmp < 0) {
            disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
            return false;
        }
    }
    return false;
}

auto BPlusTree::Begin() -> IndexIterator {
    auto *raw_cursor_page = disk_manager_->FetchPage(root_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
    while (!cursor_page->IsLeafPage()) {
        auto *internal_page = static_cast<BPlusTreeInternalPage *>(cursor_page);
        size_t next_page_id = internal_page->ValueAt(0);
        disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
        raw_cursor_page = disk_manager_->FetchPage(next_page_id);
        cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
    }
    IndexIterator iterator = IndexIterator(disk_manager_, cursor_page->GetPageId(), 0);
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
    return iterator;
}

auto BPlusTree::Begin(const KeyType &key) -> IndexIterator {
    auto *raw_cursor_page = disk_manager_->FetchPage(root_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
    while (!cursor_page->IsLeafPage()) {
        auto *internal_page = static_cast<BPlusTreeInternalPage *>(cursor_page);
        int size = internal_page->GetSize();
        int page_id = internal_page->GetPageId();
        for (int index = 1; index < size; index++) {
            if (comparator_(key, internal_page->KeyAt(index)) < 0) {
                size_t next_page_id = internal_page->ValueAt(index - 1);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
                break;
            }
            if (index == internal_page->GetSize() - 1) {
                size_t next_page_id = internal_page->ValueAt(index);
                disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
                raw_cursor_page = disk_manager_->FetchPage(next_page_id);
                cursor_page = reinterpret_cast<BPlusTreePage *>(raw_cursor_page->GetData());
            }
        }
    }
    auto *leaf_page = static_cast<BPlusTreeLeafPage *>(cursor_page);
    for (int index = 0; index < leaf_page->GetSize(); index++) {
        int cmp = comparator_(key, leaf_page->KeyAt(index));
        if (cmp <= 0) {
            IndexIterator iterator = IndexIterator(disk_manager_, leaf_page->GetPageId(), index);
            disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
            return iterator;
        }
    }
    IndexIterator iterator = IndexIterator(disk_manager_, leaf_page->GetPageId(), leaf_page->GetSize());
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
    return iterator;
}

}