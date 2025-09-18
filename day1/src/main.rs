fn main() {
    println!("Hello, world!");
}

fn exercise_example_solution() -> u64 {
    42
}

// all of this is executed when running `cargo test`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exercise_example_test() {
        let result = exercise_example_solution();
        assert_eq!(result, 42);
    }
}