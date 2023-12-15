use std::mem;
use std::time::Instant;
use rand::Rng;
use mini_rayon::join::*;

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

fn quick_sort_parallel<T>(v: &'static mut [T])
where
    T: PartialOrd + Send + 'static
{
    if v.len() <= 1 {
        return;
    }
    let mid = partition(v);
    let (lo, hi) = v.split_at_mut(mid);
    join(|| quick_sort_parallel(lo), || quick_sort_parallel(hi));
}

fn sort_benchmark(data: &Vec<u32>) {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let start_time = Instant::now();

    quick_sort(&mut data);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("sort benchmark duration: {} μs", elapsed_time.as_micros());

    assert_eq!(sorted_data, data);
}

fn sort_parallel_benchmark(data: &Vec<u32>) {
    let mut sorted_data = data.clone();
    sorted_data.sort();
    let mut data = data.clone();
    let data_ref: &mut [u32] = &mut data;
    let data_static: &'static mut [u32] = unsafe { mem::transmute(data_ref) };
    let start_time = Instant::now();

    quick_sort_parallel(data_static);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("sort parallel benchmark duration: {} μs", elapsed_time.as_micros());

    assert_eq!(sorted_data, data);
}

fn main() {
    let n = 100 * 1024 * 1024;
    let min = 1;
    let max = 100000000;
    let mut rng = rand::thread_rng();
    let random_numbers = (0..n)
        .map(|_| rng.gen_range(min..=max))
        .collect();
    let _ = sort_benchmark(&random_numbers);
    let _ = sort_parallel_benchmark(&random_numbers);
}