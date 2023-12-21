use std::mem;
use std::thread;
use std::time::Instant;
use num_cpus;
use rand::Rng;
use mini_rayon::join::join;
use threadpool::ThreadPool;

fn partition<T>(v: &mut [T]) -> usize
where
    T: PartialOrd + Send + 'static
{
    let pivot = v.len() - 1;
    let mut i = 0;
    for j in 0..pivot {
        if v[j] <= v[pivot] {
            v.swap(i, j);
            i += 1;
        }
    }
    v.swap(i, pivot);
    i
}

fn quick_sort<T>(v: &mut [T])
where
    T: PartialOrd + Send + 'static
{
    if v.len() <= 1 {
        return;
    }
    let mid = partition(v);
    let (lo, hi) = v.split_at_mut(mid);
    quick_sort(lo);
    quick_sort(hi);
}

fn quick_sort_parallel_by_spawn<T>(v: &'static mut [T])
where
    T: PartialOrd + Send + 'static
{
    if v.len() <= 200 {
        quick_sort(v);
        return;
    }
    let mid = partition(v);
    let (lo, hi) = v.split_at_mut(mid);
    let lo_handle = thread::spawn(|| quick_sort_parallel_by_spawn(lo));
    let hi_handle = thread::spawn(|| quick_sort_parallel_by_spawn(hi));
    lo_handle.join().unwrap();
    hi_handle.join().unwrap();
}

fn quick_sort_parallel_by_threadpool<T>(pool: &ThreadPool, v: &'static mut [T])
where
    T: PartialOrd + Send + 'static
{
    if v.len() <= 200 {
        quick_sort(v);
        return;
    }
    let mid = partition(v);
    let (lo, hi) = v.split_at_mut(mid);
    pool.execute(|| quick_sort_parallel_by_spawn(lo));
    pool.execute(|| quick_sort_parallel_by_spawn(hi));
    pool.join();
}

fn quick_sort_parallel_by_mini_rayon<T>(v: &'static mut [T])
where
    T: PartialOrd + Send + 'static
{
    if v.len() <= 200 {
        quick_sort(v);
        return;
    }
    let mid = partition(v);
    let (lo, hi) = v.split_at_mut(mid);
    join(|| quick_sort_parallel_by_mini_rayon(lo), || quick_sort_parallel_by_mini_rayon(hi));
}

fn sort_benchmark(data: &Vec<u32>) -> u128 {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let start_time = Instant::now();

    quick_sort(&mut data);

    let end_time = Instant::now();
    let elapsed_time: std::time::Duration = end_time.duration_since(start_time);
    assert_eq!(sorted_data, data);
    return elapsed_time.as_micros();
}

fn sort_parallel_by_spawn_benchmark(data: &Vec<u32>) -> u128 {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let data_ref: &mut [u32] = &mut data;
    let data_static: &'static mut [u32] = unsafe { mem::transmute(data_ref) };
    let start_time = Instant::now();

    quick_sort_parallel_by_spawn(data_static);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    assert_eq!(sorted_data, data);
    return elapsed_time.as_micros();
}

fn sort_parallel_by_threadpool_benchmark(data: &Vec<u32>) -> u128 {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let data_ref: &mut [u32] = &mut data;
    let data_static: &'static mut [u32] = unsafe { mem::transmute(data_ref) };
    let num_threads = num_cpus::get();
    let pool = ThreadPool::new(num_threads);
    let start_time = Instant::now();

    quick_sort_parallel_by_threadpool(&pool, data_static);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    assert_eq!(sorted_data, data);
    return elapsed_time.as_micros();
}

fn sort_parallel_by_mini_rayon_benchmark(data: &Vec<u32>) -> u128 {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let data_ref: &mut [u32] = &mut data;
    let data_static: &'static mut [u32] = unsafe { mem::transmute(data_ref) };
    let start_time = Instant::now();

    quick_sort_parallel_by_mini_rayon(data_static);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    assert_eq!(sorted_data, data);
    return elapsed_time.as_micros();
}

fn main() {
    let epoch = 4;
    let num = 1000000;
    let min = 1;
    let max = 100000000;
    let mut result_a = Vec::with_capacity(epoch);
    let mut result_b = Vec::with_capacity(epoch);
    let mut result_c = Vec::with_capacity(epoch);
    let mut result_d = Vec::with_capacity(epoch);
    for _ in 0..epoch {
        let mut rng = rand::thread_rng();
        let random_numbers = (0..num)
            .map(|_| rng.gen_range(min..=max))
            .collect();
        result_a.push(sort_benchmark(&random_numbers));
        result_b.push(sort_parallel_by_spawn_benchmark(&random_numbers));
        result_c.push(sort_parallel_by_threadpool_benchmark(&random_numbers));
        result_d.push(sort_parallel_by_mini_rayon_benchmark(&random_numbers));
    }
    let sum_a: u128 = result_a.iter().sum();
    let sum_b: u128 = result_b.iter().sum();
    let sum_c: u128 = result_c.iter().sum();
    let sum_d: u128 = result_d.iter().sum();
    println!("sort benchmark average duration: {} ms", sum_a / epoch as u128 / 1000);
    println!("sort parallel by spawn benchmark average duration: {} ms", sum_b / epoch as u128 / 1000);
    println!("sort parallel by threadpool benchmark average duration: {} ms", sum_c / epoch as u128 / 1000);
    println!("sort parallel by mini-rayon benchmark average duration: {} ms", sum_d / epoch as u128 / 1000);
}