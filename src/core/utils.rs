pub fn format_number_with_spaces(input: &str) -> String {
    let digits: Vec<char> = input.chars().filter(|c| c.is_digit(10)).collect();
    let mut result = String::new();
    let mut counter = 0;

    for &c in digits.iter().rev() {
        if counter == 3 {
            result.push(' ');
            counter = 0;
        }
        result.push(c);
        counter += 1;
    }

    result.chars().rev().collect()
}
