pub struct Permission(pub PermissionLevel);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionLevel {
    Admin,
    User,
}