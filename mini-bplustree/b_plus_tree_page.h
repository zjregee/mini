#pragma once

#include <utility>

#include "generic_key.h"

namespace minibplustree {

#define KeyType GenericKey<32>

#define ValueType size_t

#define KeyComparator GenericComparator<32>

#define MappingType std::pair<KeyType, ValueType>

#define PAGE_HEADER_SIZE sizeof(BPlusTreePage)

static constexpr int INVALID_PAGE_ID = -1;

enum class IndexPageType { LEAF_PAGE = 0, INTERNAL_PAGE };

class BPlusTreePage {
public:
    auto IsLeafPage() const -> bool;
    auto IsRootPage() const -> bool;
    void SetPageType(IndexPageType page_type);
    auto GetSize() const -> int;
    void SetSize(int size);
    void IncreaseSize(int amount);
    auto GetMaxSize() const -> int;
    void SetMaxSize(int max_size);
    auto GetMinSize() const -> int;
    auto GetParentPageId() const -> size_t;
    void SetParentPageId(size_t parent_page_id);
    auto GetPageId() const -> size_t;
    void SetPageId(size_t page_id);

private:
    IndexPageType page_type_;
    int size_;
    int max_size_;
    size_t parent_page_id_;
    size_t page_id_;
};

}