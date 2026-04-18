/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import { observer } from "mobx-react";
import Link from "next/link";

import { useTranslation } from "@plane/i18n";
import { EditIcon, CloseIcon } from "@plane/propel/icons";
// plane imports
import { Tooltip } from "@plane/propel/tooltip";
import type { ISearchIssueResponse } from "@plane/types";
import { cn } from "@plane/utils";
// hooks
import { useIssueDetail } from "@/hooks/store/use-issue-detail";
import { useProject } from "@/hooks/store/use-project";
import { usePlatformOS } from "@/hooks/use-platform-os";
// plane web components
import { IssueIdentifier } from "@/plane-web/components/issues/issue-details/issue-identifier";
// local imports
import { ParentIssuesListModal } from "../parent-issues-list-modal";

type TIssueParentSelect = {
  className?: string;
  disabled?: boolean;
  issueId: string;
  projectId: string;
  workspaceSlug: string;
  handleParentIssue: (_issueId?: string | null) => Promise<void>;
  handleRemoveSubIssue: (
    workspaceSlug: string,
    projectId: string,
    parentIssueId: string,
    issueId: string
  ) => Promise<void>;
  workItemLink: string;
};

export const IssueParentSelect = observer(function IssueParentSelect(props: TIssueParentSelect) {
  const {
    className = "",
    disabled = false,
    issueId,
    projectId,
    workspaceSlug,
    handleParentIssue,
    handleRemoveSubIssue,
    workItemLink,
  } = props;
  const { t } = useTranslation();
  // store hooks
  const { getProjectById } = useProject();
  const {
    issue: { getIssueById },
  } = useIssueDetail();
  const { isParentIssueModalOpen, toggleParentIssueModal } = useIssueDetail();

  // derived values
  const currentIssue = getIssueById(issueId);
  const parentIssue = currentIssue?.parent_id ? getIssueById(currentIssue.parent_id) : undefined;
  const parentIssueProjectDetails =
    parentIssue && parentIssue.project_id ? getProjectById(parentIssue.project_id) : undefined;
  const { isMobile } = usePlatformOS();

  if (!currentIssue) return <></>;

  return (
    <>
      <ParentIssuesListModal
        projectId={projectId}
        issueId={issueId}
        isOpen={isParentIssueModalOpen === issueId}
        handleClose={() => toggleParentIssueModal(null)}
        onChange={(selectedIssue: ISearchIssueResponse) => handleParentIssue(selectedIssue?.id)}
      />
      <button
        type="button"
        className={cn(
          "group flex items-center justify-between gap-2 rounded-sm px-2 py-0.5 outline-none",
          {
            "cursor-not-allowed": disabled,
            "hover:bg-layer-transparent-hover": !disabled,
            "bg-layer-transparent-selected": isParentIssueModalOpen,
          },
          className
        )}
        onClick={() => toggleParentIssueModal(currentIssue.id)}
        disabled={disabled}
      >
        {currentIssue.parent_id && parentIssue ? (
          <div className="flex items-center gap-1.5">
            <Tooltip tooltipHeading="Title" tooltipContent={parentIssue.name} isMobile={isMobile}>
              <Link href={workItemLink} target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()}>
                {parentIssue?.project_id && parentIssueProjectDetails && (
                  <IssueIdentifier
                    projectId={parentIssue.project_id}
                    issueTypeId={parentIssue.type_id}
                    projectIdentifier={parentIssueProjectDetails?.identifier}
                    issueSequenceId={parentIssue.sequence_id}
                    size="xs"
                    variant="secondary"
                  />
                )}
              </Link>
            </Tooltip>

            {!disabled && (
              <Tooltip tooltipContent={t("common.remove")} position="bottom" isMobile={isMobile}>
                <button
                  type="button"
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    handleRemoveSubIssue(workspaceSlug, projectId, parentIssue.id, issueId);
                  }}
                >
                  <CloseIcon className="h-2.5 w-2.5 text-tertiary hover:text-danger-primary" />
                </button>
              </Tooltip>
            )}
          </div>
        ) : (
          <span className="text-body-xs-medium text-placeholder">{t("issue.add.parent")}</span>
        )}
        {!disabled && (
          <span
            className={cn("flex-shrink-0 p-1 opacity-0 group-hover:opacity-100", {
              "text-placeholder": !currentIssue.parent_id && !parentIssue,
            })}
          >
            <EditIcon className="h-2.5 w-2.5 flex-shrink-0" />
          </span>
        )}
      </button>
    </>
  );
});
