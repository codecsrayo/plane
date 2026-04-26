/**
 * Constantes de roles extraídas de `src/auth/permissions.rs`.
 * ROLE_GUEST=5, ROLE_VIEWER=10, ROLE_MEMBER=15, ROLE_ADMIN=20
 */
export const ROLES = {
  GUEST: 5,
  VIEWER: 10,
  MEMBER: 15,
  ADMIN: 20,
} as const;

export type Role = (typeof ROLES)[keyof typeof ROLES];

/** Todos los valores válidos de rol (para usar en `expect([...]).toContain(role)`) */
export const VALID_ROLES: Role[] = [ROLES.GUEST, ROLES.VIEWER, ROLES.MEMBER, ROLES.ADMIN];
