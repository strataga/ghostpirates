use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::user::value_objects::Email;

/// User data for persistence
///
/// Simple struct for user CRUD operations
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub company_id: Uuid,
    pub email: Email,
    pub password_hash: String,
    pub full_name: String,
    pub is_active: bool,
}

/// Repository trait for User aggregate
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Create a new user
    async fn create(&self, user: User) -> Result<Uuid, String>;

    /// Find a user by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;

    /// Find a user by email address
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, String>;

    /// Find all users for a company
    async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<User>, String>;

    /// Update user's last login timestamp
    async fn update_last_login(&self, user_id: Uuid) -> Result<(), String>;
}
