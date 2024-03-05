#![feature(test)]
extern crate test;

pub fn play_game(n: u32) {
    println!("{}", fizz_buzz_fibonacci(n));
}

fn is_fibonacci_number(n: u32) -> bool {
    let (mut previous, mut current) = (0, 1);
    while current < n && n <= 2971215073 {
        let next = previous + current;
        previous = current;
        current = next;
    }
    current == n
}

pub fn fizz_buzz_fibonacci(n: u32) -> String {
    if is_fibonacci_number(n) {
        "Fibonacci".to_string()
    } else {
        match (n % 3, n % 5) {
            (0, 0) => "FizzBuzz".to_string(),
            (0, _) => "Fizz".to_string(),
            (_, 0) => "Buzz".to_string(),
            (_, _) => n.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fibonacci_number() {
        assert!(is_fibonacci_number(2_971_215_073));
        assert_eq!(is_fibonacci_number(u32::MAX), false);
    }

    #[test]
    fn test_fizz_buzz_fibonacci() {
        assert_eq!(fizz_buzz_fibonacci(1), "Fibonacci");
        assert_eq!(fizz_buzz_fibonacci(2), "Fibonacci");
        assert_eq!(fizz_buzz_fibonacci(3), "Fibonacci");
        assert_eq!(fizz_buzz_fibonacci(4), "4");
        assert_eq!(fizz_buzz_fibonacci(5), "Fibonacci");
        assert_eq!(fizz_buzz_fibonacci(6), "Fizz");
        assert_eq!(fizz_buzz_fibonacci(7), "7");
        assert_eq!(fizz_buzz_fibonacci(8), "Fibonacci");
        assert_eq!(fizz_buzz_fibonacci(9), "Fizz");
        assert_eq!(fizz_buzz_fibonacci(10), "Buzz");
        assert_eq!(fizz_buzz_fibonacci(15), "FizzBuzz");
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::hint::black_box;
    use test::Bencher;

    #[bench]
    fn bench_play_game(b: &mut Bencher) {
        b.iter(|| {
            black_box(for i in 1..=100 {
                play_game(i)
            });
        });
    }

    #[bench]
    fn bench_play_game_100(b: &mut Bencher) {
        b.iter(|| std::hint::black_box(play_game(100)));
    }

    #[bench]
    fn bench_play_game_1_000_000(b: &mut Bencher) {
        b.iter(|| std::hint::black_box(play_game(1_000_000)));
    }

    #[bench]
    fn bench_play_game_2_971_215_073(b: &mut Bencher) {
        b.iter(|| std::hint::black_box(play_game(2_971_215_073)));
    }
}
