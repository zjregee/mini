#include "b_plus_tree_leaf_page.h"

namespace minibplustree {

void BPlusTreeLeafPage::Init(size_t page_id, size_t parent_id, int max_size) {
    SetPageType(IndexPageType::LEAF_PAGE);
    SetSize(0);
    SetMaxSize(max_size);
    SetParentPageId(parent_id);
    SetPageId(page_id);
}

auto BPlusTreeLeafPage::GetNextPageId() const -> size_t {
    return next_page_id_;
}

void BPlusTreeLeafPage::SetNextPageId(size_t next_page_id) {
    next_page_id_ = next_page_id;
}

auto BPlusTreeLeafPage::KeyAt(int index) const -> KeyType {
    return array_[index].first;
}

void BPlusTreeLeafPage::SetKeyAt(int index, const KeyType &key) {
    array_[index].first = key;
}

auto BPlusTreeLeafPage::ValueAt(int index) const -> ValueType {
    return array_[index].second;
}

void BPlusTreeLeafPage::SetValueAt(int index, const ValueType &value) {
    array_[index].second = value;
}

}