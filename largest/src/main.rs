fn largest1<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut best = list[0];
    for &item in list {
        if item > best {
            best = item
        }
    }
    return best
}

fn largest_ref<T: PartialOrd>(list: &[T]) -> &T {
    let mut best = &list[0];
    for i in 0..list.len() {
        if &list[i] > best {
            best = &list[i];
        }
    }
    return best
}

fn main() {
    println!("Hello, world!");
}
