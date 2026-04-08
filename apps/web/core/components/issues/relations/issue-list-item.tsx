/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import { observer } from "mobx-react";
import { useTranslation } from "@plane/i18n";
import { LinkIcon, EditIcon, TrashIcon, CloseIcon } from "@plane/propel/icons";
// plane imports
import { Tooltip } from "@plane/propel/tooltip";
import type { TIssue, TIssueServiceType } from "@plane/types";
import { EIssueServiceType } from "@plane/types";
import { ControlLink, CustomMenu } from "@plane/ui";
import { generateWorkItemLink } from "@plane/utils";
// hooks
import { useIssueDetail } from "@/hooks/store/use-issue-detail";
import { useProject } from "@/hooks/store/use-project";
import useIssuePeekOverviewRedirection from "@/hooks/use-issue-peek-overview-redirection";
import { usePlatformOS } from "@/hooks/use-platform-os";
// plane web imports
import { IssueIdentifier } from "@/plane-web/components/issues/issue-details/issue-identifier";
import type { TIssueRelationTypes } from "@/plane-web/types";
// local imports
import { useRelationOperations } from "../issue-detail-widgets/relations/helper";
import { RelationIssueProperty } from "./properties";

type Props = {
  workspaceSlug: string;
  issueId: string;
  relationKey: TIssueRelationTypes;
  relationIssueId: string;
  disabled: boolean;
  handleIssueCrudState: (
    key: "update" | "delete" | "removeRelation",
    issueId: string,
    issue?: TIssue | null,
    relationKey?: TIssueRelationTypes | null,
    relationIssueId?: string | null
  ) => void;
  issueServiceType?: TIssueServiceType;
};

export const RelationIssueListItem = observer(function RelationIssueListItem(props: Props) {
  const {
    workspaceSlug,
    issueId,
    relationKey,
    relationIssueId,
    disabled = false,
    handleIssueCrudState,
    issueServiceType = EIssueServiceType.ISSUES,
  } = props;

  const { t } = useTranslation();

  // store hooks
  const {
    issue: { getIssueById },
    removeRelation,
    toggleCreateIssueModal,
    toggleDeleteIssueModal,
  } = useIssueDetail(issueServiceType);
  const project = useProject();
  const { isMobile } = usePlatformOS();
  // derived values
  const relatedIssue = getIssueById(relationIssueId);
  const { handleRedirection } = useIssuePeekOverviewRedirection(!!relatedIssue?.is_epic);
  const issueOperations = useRelationOperations(
    relatedIssue?.is_epic ? EIssueServiceType.EPICS : EIssueServiceType.ISSUES
  );
  const projectDetail =
    (relatedIssue && relatedIssue.project_id && project.getProjectById(relatedIssue.project_id)) || undefined;
  const projectId = relatedIssue?.project_id;

  if (!relatedIssue || !projectId) return <></>;

  const workItemLink = generateWorkItemLink({
    workspaceSlug: workspaceSlug.toString(),
    projectId: relatedIssue.project_id,
    issueId: relatedIssue.id,
    projectIdentifier: projectDetail?.identifier,
    sequenceId: relatedIssue.sequence_id,
    isEpic: relatedIssue.is_epic,
  });

  // handlers
  const handleIssuePeekOverview = (selectedIssue: TIssue) => {
    if (selectedIssue.is_epic) {
      // open epics in new tab
      window.open(workItemLink, "_blank");
      return;
    }
    handleRedirection(workspaceSlug, selectedIssue, isMobile);
  };

  const handleEditIssue = (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
    e.stopPropagation();
    e.preventDefault();
    handleIssueCrudState("update", relationIssueId, { ...relatedIssue });
    toggleCreateIssueModal(true);
  };

  const handleDeleteIssue = (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
    e.stopPropagation();
    e.preventDefault();
    handleIssueCrudState("delete", relationIssueId, relatedIssue);
    toggleDeleteIssueModal(relationIssueId);
    handleIssueCrudState("removeRelation", issueId, relatedIssue, relationKey, relationIssueId);
  };

  const handleCopyIssueLink = (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
    e.stopPropagation();
    e.preventDefault();
    issueOperations.copyLink(workItemLink);
  };

  const handleRemoveRelation = (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
    e.preventDefault();
    e.stopPropagation();
    removeRelation(workspaceSlug, projectId, issueId, relationKey, relationIssueId);
  };

  return (
    <div key={relationIssueId}>
      <ControlLink
        id={`issue-${relatedIssue.id}`}
        href={workItemLink}
        onClick={() => handleIssuePeekOverview(relatedIssue)}
        className="w-full cursor-pointer"
      >
        {relatedIssue && (
          <div className="group relative flex h-full min-h-11 w-full items-center px-1.5 py-1 transition-all hover:bg-surface-2">
            <span className="size-5 flex-shrink-0" />
            <div className="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
              <div className="flex-shrink-0">
                {projectDetail && (
                  <IssueIdentifier
                    projectId={projectDetail.id}
                    issueTypeId={relatedIssue.type_id}
                    projectIdentifier={projectDetail.identifier}
                    issueSequenceId={relatedIssue.sequence_id}
                    size="xs"
                    variant="secondary"
                  />
                )}
              </div>

              <Tooltip tooltipContent={relatedIssue.name} isMobile={isMobile}>
                <span className="w-0 flex-1 truncate text-13 text-primary">{relatedIssue.name}</span>
              </Tooltip>
            </div>
            <div
              role="presentation"
              className="flex-shrink-0 text-13"
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
              }}
            >
              <RelationIssueProperty
                workspaceSlug={workspaceSlug}
                issueId={relationIssueId}
                disabled={disabled}
                issueOperations={issueOperations}
                issueServiceType={issueServiceType}
              />
            </div>
            <div className="flex-shrink-0 pl-2 text-13">
              <CustomMenu placement="bottom-end" ellipsis>
                {!disabled && (
                  <CustomMenu.MenuItem onClick={handleEditIssue}>
                    <div className="flex items-center gap-2">
                      <EditIcon className="h-3.5 w-3.5" strokeWidth={2} />
                      <span>{t("common.actions.edit")}</span>
                    </div>
                  </CustomMenu.MenuItem>
                )}

                <CustomMenu.MenuItem onClick={handleCopyIssueLink}>
                  <div className="flex items-center gap-2">
                    <LinkIcon className="h-3.5 w-3.5" strokeWidth={2} />
                    <span>{t("common.actions.copy_link")}</span>
                  </div>
                </CustomMenu.MenuItem>

                {!disabled && (
                  <CustomMenu.MenuItem onClick={handleRemoveRelation}>
                    <div className="flex items-center gap-2">
                      <CloseIcon className="h-3.5 w-3.5" strokeWidth={2} />
                      <span>{t("common.actions.remove_relation")}</span>
                    </div>
                  </CustomMenu.MenuItem>
                )}

                {!disabled && (
                  <CustomMenu.MenuItem onClick={handleDeleteIssue}>
                    <div className="flex items-center gap-2">
                      <TrashIcon className="h-3.5 w-3.5" strokeWidth={2} />
                      <span>{t("common.actions.delete")}</span>
                    </div>
                  </CustomMenu.MenuItem>
                )}
              </CustomMenu>
            </div>
          </div>
        )}
      </ControlLink>
    </div>
  );
});
