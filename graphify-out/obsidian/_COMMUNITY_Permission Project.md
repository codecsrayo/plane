---
type: community
cohesion: 0.06
members: 49
---

# Permission Project

**Cohesion:** 0.06 - loosely connected
**Members:** 49 nodes

## Members

- [[TODO Move the below logic to python match - python v3.10]] - rationale - api/plane/utils/permissions/workspace.py
- [[._check_access_and_get_role()]] - code - api/plane/utils/permissions/page.py
- [[._check_project_action_access()]] - code - api/plane/utils/permissions/page.py
- [[._check_project_member_access()]] - code - api/plane/utils/permissions/page.py
- [[._has_private_page_action_access()]] - code - api/plane/utils/permissions/page.py
- [[._has_public_page_action_access()]] - code - api/plane/utils/permissions/page.py
- [[.has_permission()_12]] - code - api/plane/license/api/permissions/instance.py
- [[.has_permission()_6]] - code - api/plane/utils/permissions/page.py
- [[.has_permission()_10]] - code - api/plane/utils/permissions/project.py
- [[.has_permission()_7]] - code - api/plane/utils/permissions/project.py
- [[.has_permission()_9]] - code - api/plane/utils/permissions/project.py
- [[.has_permission()_11]] - code - api/plane/utils/permissions/project.py
- [[.has_permission()_8]] - code - api/plane/utils/permissions/project.py
- [[.has_permission()_2]] - code - api/plane/utils/permissions/workspace.py
- [[.has_permission()]] - code - api/plane/utils/permissions/workspace.py
- [[.has_permission()_3]] - code - api/plane/utils/permissions/workspace.py
- [[.has_permission()_1]] - code - api/plane/utils/permissions/workspace.py
- [[.has_permission()_5]] - code - api/plane/utils/permissions/workspace.py
- [[.has_permission()_4]] - code - api/plane/utils/permissions/workspace.py
- [[BasePermission]] - code
- [[Check access to private pages. Override for feature flag logic.]] - rationale - api/plane/utils/permissions/page.py
- [[Check basic project-level permissions before checking object-level permissions.]] - rationale - api/plane/utils/permissions/page.py
- [[Check if the user has permission to access a public page         and can perform]] - rationale - api/plane/utils/permissions/page.py
- [[Check if the user is a project member.]] - rationale - api/plane/utils/permissions/page.py
- [[Custom permission to control access to pages within a workspace     based on use]] - rationale - api/plane/utils/permissions/page.py
- [[Hook for extended access checking         Returns True (allow), False (deny), N]] - rationale - api/plane/utils/permissions/page.py
- [[InstanceAdminPermission]] - code - api/plane/license/api/permissions/instance.py
- [[ProjectAdminPermission]] - code - api/plane/utils/permissions/project.py
- [[ProjectBasePermission]] - code - api/plane/utils/permissions/project.py
- [[ProjectEntityPermission]] - code - api/plane/utils/permissions/project.py
- [[ProjectLitePermission]] - code - api/plane/utils/permissions/project.py
- [[ProjectMemberPermission]] - code - api/plane/utils/permissions/project.py
- [[ProjectPagePermission]] - code - api/plane/utils/permissions/page.py
- [[WorkSpaceAdminPermission]] - code - api/plane/utils/permissions/workspace.py
- [[WorkSpaceBasePermission]] - code - api/plane/utils/permissions/workspace.py
- [[WorkspaceEntityPermission]] - code - api/plane/utils/permissions/workspace.py
- [[WorkspaceOwnerPermission]] - code - api/plane/utils/permissions/workspace.py
- [[WorkspaceUserPermission]] - code - api/plane/utils/permissions/workspace.py
- [[WorkspaceViewerPermission]] - code - api/plane/utils/permissions/workspace.py
- [[__init__.py_5]] - code - api/plane/app/permissions/**init**.py
- [[__init__.py_56]] - code - api/plane/license/api/permissions/**init**.py
- [[__init__.py_36]] - code - api/plane/utils/permissions/**init**.py
- [[instance.py_1]] - code - api/plane/license/api/permissions/instance.py
- [[page.py]] - code - api/plane/app/permissions/page.py
- [[page.py_4]] - code - api/plane/utils/permissions/page.py
- [[project.py]] - code - api/plane/app/permissions/project.py
- [[project.py_7]] - code - api/plane/utils/permissions/project.py
- [[workspace.py]] - code - api/plane/app/permissions/workspace.py
- [[workspace.py_5]] - code - api/plane/utils/permissions/workspace.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Permission_Project
SORT file.name ASC
```

## Connections to other communities

- 7 edges to [[_COMMUNITY_Project Return]]
- 3 edges to [[_COMMUNITY_Estimate Endpoint]]
- 1 edge to [[_COMMUNITY_Instance Endpoint]]

## Top bridge nodes

- [[ProjectEntityPermission]] - degree 8, connects to 2 communities
- [[ProjectAdminPermission]] - degree 5, connects to 1 community
- [[ProjectBasePermission]] - degree 5, connects to 1 community
- [[ProjectLitePermission]] - degree 5, connects to 1 community
- [[ProjectMemberPermission]] - degree 5, connects to 1 community
