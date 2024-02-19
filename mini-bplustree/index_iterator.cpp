#include "index_iterator.h"

namespace minibplustree {

IndexIterator::IndexIterator(DiskManager *disk_manager, size_t start_page_id, size_t start_index) : disk_manager_(disk_manager), current_page_id_(start_page_id), current_index_(start_index) {
    auto *raw_cursor_page = disk_manager_->FetchPage(current_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_cursor_page->GetData());
    current_data_.first = cursor_page->KeyAt(current_index_);
    current_data_.second = cursor_page->ValueAt(current_index_);
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
}

auto IndexIterator::IsEnd() -> bool {
    auto *raw_cursor_page = disk_manager_->FetchPage(current_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_cursor_page->GetData());
    bool is_end = current_index_ == (cursor_page->GetSize() - 1) && cursor_page->GetNextPageId() == INVALID_PAGE_ID;
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
    return is_end;
}

auto IndexIterator::operator*() -> const MappingType & {
    return current_data_;
}

auto IndexIterator::operator++() -> IndexIterator & {
    auto *raw_cursor_page = disk_manager_->FetchPage(current_page_id_);
    auto *cursor_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_cursor_page->GetData());
    if (current_index_ == (cursor_page->GetSize() - 1)) {
        current_page_id_ = cursor_page->GetNextPageId();
        current_index_ = 0;
        disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
        raw_cursor_page = disk_manager_->FetchPage(current_page_id_);
        cursor_page = reinterpret_cast<BPlusTreeLeafPage *>(raw_cursor_page->GetData());
    } else {
        current_index_ += 1;
    }
    current_data_.first = cursor_page->KeyAt(current_index_);
    current_data_.second = cursor_page->ValueAt(current_index_);
    disk_manager_->UnpinPage(cursor_page->GetPageId(), raw_cursor_page, false);
    return *this;
}

auto IndexIterator::operator==(const IndexIterator &itr) const -> bool {
    return current_page_id_ == itr.current_page_id_ && current_index_ == itr.current_index_;
}

auto IndexIterator::operator!=(const IndexIterator &itr) const -> bool {
    return current_page_id_ != itr.current_page_id_ || current_index_ != itr.current_index_;
}

}