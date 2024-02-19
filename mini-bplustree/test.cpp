#include <vector>
#include <chrono>
#include <random>
#include <utility>
#include <iomanip>
#include <sstream>
#include <iostream>
#include <unordered_map>
#include <algorithm>

#include "b_plus_tree.h"
#include "disk_manager.h"

auto generate_random_data(size_t data_num, size_t key_size) -> std::vector<std::pair<std::string, size_t>> {
    std::vector<std::pair<std::string, size_t>> random_data;
    random_data.reserve(data_num);

    const std::string charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    std::random_device rd;
    std::mt19937 generator(rd());
    std::uniform_int_distribution<int> distribution_1(0, charset.size() - 1);
    std::uniform_int_distribution<size_t> distribution_2(0, std::numeric_limits<size_t>::max());

    std::unordered_map<std::string, bool> random_keys_map;
    for (size_t i = 0; i < data_num; i++) {
        while (true) {
            std::string random_key;
            random_key.reserve(key_size);
            for (size_t j = 0; j < key_size; j++) {
                random_key.push_back(charset[distribution_1(generator)]);
            }
            if (!random_keys_map.count(random_key)) {
                random_data.push_back(std::make_pair(random_key, distribution_2(generator)));
                random_keys_map[random_key] = true;
                break;
            }
        }
    }

    return random_data;
}

auto generate_sequential_data(size_t data_num, size_t key_size) -> std::vector<std::pair<std::string, size_t>> {
    std::vector<std::pair<std::string, size_t>> sequential_data;
    sequential_data.reserve(data_num);

    for (size_t i = 0; i < data_num; i++) {
        std::stringstream ss;
        ss << std::right << std::setw(key_size) << std::setfill('0') << i;
        sequential_data.push_back(std::make_pair(ss.str(), i));
    }

    return sequential_data;
}

void print_data(minibplustree::BPlusTree *index) {

}

int main(int argc, char* argv[]) {
    size_t data_num;
    size_t key_size = 32;

    if (argc < 2) {
        data_num = 8000;
    } else {
        data_num = std::atoi(argv[1]);
    }
    
    std::vector<std::pair<std::string, size_t>> data = generate_sequential_data(data_num, key_size);

    std::random_device rd;
    std::mt19937 g(rd());
    std::shuffle(data.begin(), data.end(), g);

    minibplustree::DiskManager *disk = new minibplustree::DiskManager("sim_disk");
    minibplustree::BPlusTree *index = new minibplustree::BPlusTree(disk);
    
    {
        auto start_time = std::chrono::high_resolution_clock::now();

        for (size_t i = 0; i < data_num; i++) {
            // if (i % 100 == 0) {
            //     std::cout << "BPlusTree execute " << i << "th insert" << std::endl;
            // }
            minibplustree::KeyType key = minibplustree::KeyType{};
            std::memcpy(key.data_, data[i].first.c_str(), key_size);
            if (!index->Insert(key, data[i].second)) {
                std::cout << "BPlusTree execute " << i << "th insert error: can' insert key" << std::endl;
            }
        }

        auto end_time = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);
        std::cout << "BPlusTree insert duration: " << duration.count() << "ms" << std::endl;
    }

    print_data(index);

    {
        auto start_time = std::chrono::high_resolution_clock::now();

        for (size_t i = 0; i < data_num; i++) {
            minibplustree::KeyType key = minibplustree::KeyType{};
            std::memcpy(key.data_, data[i].first.c_str(), key_size);
            size_t value;
            if (!index->GetValue(key, &value)) {
                std::cout << "BPlusTree execute " << i << "th queury error: can' find key" << std::endl;
                break;
            }
            if (value != data[i].second) {
                std::cout << "BPlusTree execute " << i << "th queury error: correct value - " <<  data[i].second << " get value - " << value << std::endl;
                break;
            }
        }

        auto end_time = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);
        std::cout << "BPlusTree query duration: " << duration.count() << "ms" << std::endl;
    }

    return 0;
}