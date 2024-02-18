#include "b_plus_tree_internal_page.h"

namespace minibplustree {

void BPlusTreeInternalPage::Init(size_t page_id, size_t parent_id, int max_size) {
    SetPageType(IndexPageType::INTERNAL_PAGE);
    SetSize(0);
    SetMaxSize(max_size);
    SetParentPageId(parent_id);
    SetPageId(page_id);
}

auto BPlusTreeInternalPage::KeyAt(int index) const -> KeyType {
    return array_[index].first;
}

void BPlusTreeInternalPage::SetKeyAt(int index, const KeyType &key) {
    array_[index].first = key;
}

auto BPlusTreeInternalPage::ValueAt(int index) const -> ValueType {
    return array_[index].second;
}

void BPlusTreeInternalPage::SetValueAt(int index, const ValueType &value) {
    array_[index].second = value;
}

}