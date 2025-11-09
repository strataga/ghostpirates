use serde::{Deserialize, Serialize};
use std::fmt;

/// Email value object representing a valid email address
///
/// # Invariants
/// - Must contain '@' character
/// - Must be at least 3 characters long
/// - Is immutable after construction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Email(String);

impl Email {
    /// Creates a new Email value object
    ///
    /// # Arguments
    /// * `email` - The email string to validate
    ///
    /// # Returns
    /// * `Ok(Email)` - If email is valid
    /// * `Err(String)` - If email is invalid
    ///
    /// # Example
    /// ```
    /// use ghostpirates_api::domain::user::value_objects::Email;
    ///
    /// let email = Email::new("test@example.com").expect("valid email");
    /// assert_eq!(email.as_str(), "test@example.com");
    /// ```
    #[allow(dead_code)]
    pub fn new(email: impl Into<String>) -> Result<Self, String> {
        let email = email.into();
        if Self::is_valid(&email) {
            Ok(Email(email))
        } else {
            Err(format!("Invalid email: {}", email))
        }
    }

    /// Validates an email string
    ///
    /// # Validation Rules
    /// - Must contain '@' character
    /// - Must be at least 3 characters long
    #[allow(dead_code)]
    fn is_valid(email: &str) -> bool {
        email.contains('@') && email.len() >= 3
    }

    /// Returns the email as a string slice
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email() {
        assert!(Email::new("test@example.com").is_ok());
    }

    #[test]
    fn valid_email_with_subdomain() {
        assert!(Email::new("user@mail.example.com").is_ok());
    }

    #[test]
    fn valid_email_minimum_length() {
        assert!(Email::new("a@b").is_ok());
    }

    #[test]
    fn invalid_email_no_at_symbol() {
        assert!(Email::new("invalid").is_err());
    }

    #[test]
    fn invalid_email_too_short() {
        assert!(Email::new("a@").is_err());
    }

    #[test]
    fn invalid_email_empty() {
        assert!(Email::new("").is_err());
    }

    #[test]
    fn email_display() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(format!("{}", email), "test@example.com");
    }

    #[test]
    fn email_clone() {
        let email1 = Email::new("test@example.com").unwrap();
        let email2 = email1.clone();
        assert_eq!(email1, email2);
    }
}
