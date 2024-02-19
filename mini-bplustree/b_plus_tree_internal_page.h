#pragma once

#include <string>
#include <sstream>

#include "b_plus_tree_page.h"

namespace minibplustree {

#define INTERNAL_PAGE_HEADER_SIZE PAGE_HEADER_SIZE

#define INTERNAL_PAGE_SIZE ((4096 - INTERNAL_PAGE_HEADER_SIZE) / (sizeof(MappingType)))

class BPlusTreeInternalPage : public BPlusTreePage {
public:
    void Init(size_t page_id, size_t parent_id = INVALID_PAGE_ID, int max_size = INTERNAL_PAGE_SIZE);
    auto KeyAt(int index) const -> KeyType;
    void SetKeyAt(int index, const KeyType &key);
    auto ValueAt(int index) const -> ValueType;
    void SetValueAt(int index, const ValueType &value);

    auto Debug() const -> std::string {
        std::ostringstream oss;
        oss << "BPlusTreeInternalPage:" << std::endl;
        oss << " page_type: " << static_cast<int>(GetPageType()) << std::endl;
        oss << " page_id: " << GetPageId() << std::endl;
        oss << " parent_page_id: " << GetParentPageId() << std::endl;
        oss << " size: " << GetSize() << std::endl;
        oss << " max_size: " << GetMaxSize() << std::endl;
        for (size_t i = 0; i < GetSize(); i++) {
            oss << " key " << i << ": " << std::string(KeyAt(i).data_, 32) << " value " << i << ": " << ValueAt(i) << std::endl;
        }
        return oss.str();
    }

private:
    MappingType array_[1];
};

}