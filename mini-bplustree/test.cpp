#include <vector>
#include <chrono>
#include <random>
#include <iostream>
#include <algorithm>

#include "b_plus_tree.h"
#include "disk_manager.h"

int main(int argc, char* argv[]) {
    size_t data_num;
    size_t key_size = 32;

    if (argc < 2) {
        data_num = 10000;
    } else {
        data_num = std::atoi(argv[1]);
    }

    std::vector<std::string> random_keys;
    std::vector<size_t> random_values;
    std::vector<std::string> random_query_keys;

    random_keys.reserve(data_num);
    random_values.reserve(data_num);
    random_query_keys.reserve(2 * data_num);

    const std::string charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    std::random_device rd;
    std::mt19937 generator(rd());
    std::uniform_int_distribution<int> distribution(0, charset.size() - 1);

    for (size_t i = 0; i < data_num; i++) {
        std::string random_key;
        std::string null_query_key;
        random_key.reserve(key_size);
        null_query_key.reserve(key_size);
        for (size_t j = 0; j < key_size; j++) {
            random_key.push_back(charset[distribution(generator)]);
        }
        for (size_t j = 0; j < key_size; j++) {
            null_query_key.push_back(charset[distribution(generator)]);
        }
        random_keys.push_back(random_key);
        random_query_keys.push_back(random_key);
        random_query_keys.push_back(null_query_key);
    }

    std::shuffle(random_query_keys.begin(), random_query_keys.end(), generator);
    
    minibplustree::DiskManager* disk = new minibplustree::DiskManager();
    minibplustree::BPlusTree* index = new minibplustree::BPlusTree(disk);
    
    {
        auto start_time = std::chrono::high_resolution_clock::now();

        for (size_t i = 0; i < 2 * data_num; i++) {

        }

        auto end_time = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);
        std::cout << "BPlusTreee 数据查询时间：" << duration.count() << "ms" << std::endl;
    }
    return 0;
}