pub struct UserMeta {
    pub discord_user_id: u64,
    pub is_admin: bool,
    pub is_moderator: bool,
    pub is_verified: bool,
    pub stripe_customer_id: Option<String>,
}
