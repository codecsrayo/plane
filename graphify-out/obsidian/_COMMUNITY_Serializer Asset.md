---
type: community
cohesion: 0.05
members: 46
---

# Serializer Asset

**Cohesion:** 0.05 - loosely connected
**Members:** 46 nodes

## Members

- [[.create()_31]] - code - api/plane/api/serializers/estimate.py
- [[.update()_31]] - code - api/plane/api/serializers/intake.py
- [[.validate()_3]] - code - api/plane/api/serializers/estimate.py
- [[.validate()_16]] - code - api/plane/api/serializers/intake.py
- [[.validate_member()]] - code - api/plane/api/serializers/member.py
- [[.validate_role()_1]] - code - api/plane/api/serializers/member.py
- [[AssetUpdateSerializer]] - code - api/plane/api/serializers/asset.py
- [[Comprehensive file asset serializer with complete metadata and URL generation.]] - rationale - api/plane/api/serializers/asset.py
- [[EstimatePointSerializer]] - code - api/plane/api/serializers/estimate.py
- [[EstimateReadSerializer]] - code - api/plane/app/serializers/estimate.py
- [[EstimateSerializer]] - code - api/plane/api/serializers/estimate.py
- [[FileAssetSerializer]] - code - api/plane/api/serializers/asset.py
- [[GenericAssetUpdateSerializer]] - code - api/plane/api/serializers/asset.py
- [[GenericAssetUploadSerializer]] - code - api/plane/api/serializers/asset.py
- [[IntakeIssueCreateSerializer]] - code - api/plane/api/serializers/intake.py
- [[IntakeIssueUpdateSerializer]] - code - api/plane/api/serializers/intake.py
- [[IssueDataSerializer]] - code - api/plane/api/serializers/intake.py
- [[IssueForIntakeSerializer]] - code - api/plane/api/serializers/intake.py
- [[Meta]] - code - api/plane/api/serializers/asset.py
- [[Meta_8]] - code - api/plane/api/serializers/estimate.py
- [[Meta_21]] - code - api/plane/api/serializers/invite.py
- [[Meta_23]] - code - api/plane/api/serializers/member.py
- [[Meta_22]] - code - api/plane/api/serializers/sticky.py
- [[ProjectMemberSerializer_1]] - code - api/plane/api/serializers/member.py
- [[Serializer for asset status updates after successful upload completion.      Han]] - rationale - api/plane/api/serializers/asset.py
- [[Serializer for creating intake work items with embedded issue data.      Manages]] - rationale - api/plane/api/serializers/intake.py
- [[Serializer for generic asset upload confirmation and status management.      Han]] - rationale - api/plane/api/serializers/asset.py
- [[Serializer for generic asset upload requests with project association.      Vali]] - rationale - api/plane/api/serializers/asset.py
- [[Serializer for nested work item data in intake request payloads.      Validates]] - rationale - api/plane/api/serializers/intake.py
- [[Serializer for project members.]] - rationale - api/plane/api/serializers/member.py
- [[Serializer for updating intake work items and their associated issues.      Hand]] - rationale - api/plane/api/serializers/intake.py
- [[Serializer for user asset upload requests.      This serializer validates the me]] - rationale - api/plane/api/serializers/asset.py
- [[Serializer for work item data within intake submissions.      Handles essential]] - rationale - api/plane/api/serializers/intake.py
- [[Update intake issue and transition associated issue state if accepted.]] - rationale - api/plane/api/serializers/intake.py
- [[UserAssetUploadSerializer]] - code - api/plane/api/serializers/asset.py
- [[Validate that if status is being changed to accepted (1),         the project ha_1]] - rationale - api/plane/api/serializers/intake.py
- [[WorkspaceEstimateSerializer]] - code - api/plane/app/serializers/estimate.py
- [[__init__.py_13]] - code - api/plane/api/serializers/**init**.py
- [[asset.py_5]] - code - api/plane/api/serializers/asset.py
- [[base.py_24]] - code - api/plane/api/serializers/base.py
- [[estimate.py_5]] - code - api/plane/api/serializers/estimate.py
- [[estimate.py_2]] - code - api/plane/app/serializers/estimate.py
- [[intake.py_4]] - code - api/plane/api/serializers/intake.py
- [[invite.py_4]] - code - api/plane/api/serializers/invite.py
- [[member.py_4]] - code - api/plane/api/serializers/member.py
- [[sticky.py_3]] - code - api/plane/api/serializers/sticky.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Serializer_Asset
SORT file.name ASC
```

## Connections to other communities

- 19 edges to [[_COMMUNITY_Serializer Issue]]
- 13 edges to [[_COMMUNITY_Serializer Project]]
- 9 edges to [[_COMMUNITY_Serializer Issue]]
- 6 edges to [[_COMMUNITY_Serializer Meta]]
- 4 edges to [[_COMMUNITY_Partial Endpoint]]
- 3 edges to [[_COMMUNITY_Meta Serializer]]
- 2 edges to [[_COMMUNITY_Serializer Issue]]
- 2 edges to [[_COMMUNITY_Serializer Member]]
- 1 edge to [[_COMMUNITY_Endpoint Workspace]]
- 1 edge to [[_COMMUNITY_Project Return]]
- 1 edge to [[_COMMUNITY_Intake Endpoint]]
- 1 edge to [[_COMMUNITY_Endpoint State]]
- 1 edge to [[_COMMUNITY_Serializer Validate]]

## Top bridge nodes

- [[base.py_24]] - degree 14, connects to 5 communities
- [[__init__.py_13]] - degree 13, connects to 5 communities
- [[IntakeIssueUpdateSerializer]] - degree 8, connects to 4 communities
- [[EstimateReadSerializer]] - degree 7, connects to 3 communities
- [[FileAssetSerializer]] - degree 5, connects to 3 communities
