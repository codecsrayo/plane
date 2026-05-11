---
type: community
cohesion: 0.23
members: 13
---

# Member Sort

**Cohesion:** 0.23 - loosely connected
**Members:** 13 nodes

## Members

- [[IMemberFilters]] - code - store/member/utils.ts
- [[MemberHeaderColumn]] - code - components/project/member-header-column.tsx
- [[Props_134]] - code - components/project/member-header-column.tsx
- [[filterProjectMembersByRole()]] - code - store/member/utils.ts
- [[filterWorkspaceMembersByRole()]] - code - store/member/utils.ts
- [[getMemberSortKey()]] - code - store/member/utils.ts
- [[member-header-column.tsx]] - code - components/project/member-header-column.tsx
- [[parseOrderKey()]] - code - store/member/utils.ts
- [[projectMemberIds()]] - code - store/member/project/base-project-member.store.ts
- [[sortMembers()]] - code - store/member/utils.ts
- [[sortProjectMembers()]] - code - store/member/utils.ts
- [[sortWorkspaceMembers()]] - code - store/member/utils.ts
- [[utils.ts]] - code - store/member/utils.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Member_Sort
SORT file.name ASC
```

## Connections to other communities

- 6 edges to [[_COMMUNITY_Member Project]]
- 3 edges to [[_COMMUNITY_Workspace Member]]

## Top bridge nodes

- [[utils.ts]] - degree 12, connects to 2 communities
- [[IMemberFilters]] - degree 4, connects to 2 communities
- [[sortProjectMembers()]] - degree 6, connects to 1 community
- [[sortWorkspaceMembers()]] - degree 4, connects to 1 community
- [[projectMemberIds()]] - degree 2, connects to 1 community
