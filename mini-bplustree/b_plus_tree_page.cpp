#include "b_plus_tree_page.h"

namespace minibplustree {

auto BPlusTreePage::IsLeafPage() const -> bool {
    return page_type_ == IndexPageType::LEAF_PAGE;
}

auto BPlusTreePage::IsRootPage() const -> bool {
    return parent_page_id_ == static_cast<size_t>(IndexPageType::INVALID_INDEX_PAGE);
}

void BPlusTreePage::SetPageType(IndexPageType page_type) {
    page_type_ = page_type;
}

auto BPlusTreePage::GetSize() const -> int {
    return size_;
}

void BPlusTreePage::SetSize(int size) {
    size_ = size;
}

void BPlusTreePage::IncreaseSize(int amount) {
    size_ += amount;
}

auto BPlusTreePage::GetMaxSize() const -> int {
    return max_size_;
}

void BPlusTreePage::SetMaxSize(int size) {
    max_size_ = size;
}

auto BPlusTreePage::GetMinSize() const -> int {
    return max_size_ / 2;
}

auto BPlusTreePage::GetParentPageId() const -> size_t {
    return parent_page_id_;
}

void BPlusTreePage::SetParentPageId(size_t parent_page_id) {
    parent_page_id_ = parent_page_id;
}

auto BPlusTreePage::GetPageId() const -> size_t {
    return page_id_;
}

void BPlusTreePage::SetPageId(size_t page_id) {
    page_id_ = page_id;
}

}