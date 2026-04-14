use crate::{entities::workspace_members, error::AppError};

pub const ROLE_GUEST: i16 = 5;
pub const ROLE_VIEWER: i16 = 10;
pub const ROLE_MEMBER: i16 = 15;
pub const ROLE_ADMIN: i16 = 20;

/// Verifica que el miembro tenga rol Admin en el workspace.
/// Equivalente a la verificación `role >= 20` usada en la API de Django.
pub fn require_workspace_admin(member: &workspace_members::Model) -> Result<(), AppError> {
    if member.role >= ROLE_ADMIN {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn require_role(
    project_role: i16,
    workspace_role: i16,
    required_role: i16,
) -> Result<(), AppError> {
    if project_role >= required_role || workspace_role >= ROLE_ADMIN {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::{require_role, ROLE_ADMIN, ROLE_MEMBER, ROLE_VIEWER};

    #[test]
    fn require_role_accepts_project_role() {
        assert!(require_role(ROLE_MEMBER, ROLE_VIEWER, ROLE_MEMBER).is_ok());
    }

    #[test]
    fn require_role_accepts_workspace_admin_override() {
        assert!(require_role(ROLE_VIEWER, ROLE_ADMIN, ROLE_MEMBER).is_ok());
    }

    #[test]
    fn require_role_rejects_insufficient_roles() {
        assert!(require_role(ROLE_VIEWER, ROLE_VIEWER, ROLE_MEMBER).is_err());
    }
}
