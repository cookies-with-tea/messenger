use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^(8|7|\+7)(\d{10}|(\s\(\d{3}\)\s\d{3}\s\d{2}\s\d{2}))$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_password(password: &str) -> bool {
    if password.len() < 6 {
        return false;
    }

    if !password.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    if password.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }

    true
}
