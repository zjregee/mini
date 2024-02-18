#pragma once

#include <stddef.h>
#include <cstring>

namespace minibplustree {

template <size_t KeySize>
class GenericKey {
public:
    char data_[KeySize];
};

template <size_t KeySize>
class GenericComparator {
public:
    inline auto operator()(const GenericKey<KeySize> &lhs, const GenericKey<KeySize> &rhs) const -> int {
        return std::memcmp(lhs.data_, rhs.data_, KeySize);
    }
};

}