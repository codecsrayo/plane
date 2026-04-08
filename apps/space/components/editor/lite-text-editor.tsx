/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
// plane imports
import { LiteTextEditorWithRef } from "@plane/editor";
import type { EditorRefApi, ILiteTextEditorProps, TFileHandler } from "@plane/editor";
import type { MakeOptional } from "@plane/types";
import { cn, isCommentEmpty } from "@plane/utils";
// helpers
import { getEditorFileHandlers } from "@/helpers/editor.helper";
// hooks
import { useParseEditorContent } from "@/hooks/use-parse-editor-content";
// plane web imports
import { useEditorFlagging } from "@/hooks/use-editor-flagging";
// local imports
import { EditorMentionsRoot } from "./embeds/mentions";
import { IssueCommentToolbar } from "./toolbar";

function isMutableRefObject<T>(forwardedRef: React.ForwardedRef<T>): forwardedRef is React.MutableRefObject<T | null> {
  return !!forwardedRef && typeof forwardedRef === "object" && "current" in forwardedRef;
}

type LiteTextEditorWrapperProps = MakeOptional<
  Omit<ILiteTextEditorProps, "fileHandler" | "mentionHandler" | "extendedEditorProps">,
  "disabledExtensions" | "flaggedExtensions" | "getEditorMetaData"
> & {
  anchor: string;
  isSubmitting?: boolean;
  showSubmitButton?: boolean;
  workspaceId: string;
} & (
    | {
        editable: false;
      }
    | {
        editable: true;
        uploadFile: TFileHandler["upload"];
      }
  );

export const LiteTextEditor = React.forwardRef(function LiteTextEditor(
  editorProps: LiteTextEditorWrapperProps,
  ref: React.ForwardedRef<EditorRefApi>
) {
  const {
    anchor,
    containerClassName,
    disabledExtensions: additionalDisabledExtensions = [],
    editable,
    isSubmitting = false,
    showSubmitButton = true,
    workspaceId,
    ...rest
  } = editorProps;
  // derived values
  const isEmpty = isCommentEmpty(editorProps.initialValue);
  const editorRef = isMutableRefObject<EditorRefApi>(ref) ? ref.current : null;
  const { liteText: liteTextEditorExtensions } = useEditorFlagging(anchor);
  // parse content
  const { getEditorMetaData } = useParseEditorContent({
    anchor,
  });

  return (
    <div className="space-y-3 rounded-sm border border-subtle p-3">
      <LiteTextEditorWithRef
        ref={ref}
        disabledExtensions={[...liteTextEditorExtensions.disabled, ...additionalDisabledExtensions]}
        flaggedExtensions={liteTextEditorExtensions.flagged}
        editable={editable}
        fileHandler={getEditorFileHandlers({
          anchor,
          uploadFile: editable ? editorProps.uploadFile : async () => "",
          workspaceId,
        })}
        getEditorMetaData={getEditorMetaData}
        mentionHandler={{
          renderComponent: (mentionProps) => <EditorMentionsRoot {...mentionProps} />,
        }}
        extendedEditorProps={{}}
        {...rest}
        // overriding the containerClassName to add relative class passed
        containerClassName={cn(containerClassName, "relative")}
      />
      {editable && (
        <IssueCommentToolbar
          executeCommand={(item) => {
            // TODO: update this while toolbar homogenization
            // @ts-expect-error type mismatch here
            editorRef?.executeMenuItemCommand({
              itemKey: item.itemKey,
              ...item.extraProps,
            });
          }}
          isSubmitting={isSubmitting}
          showSubmitButton={showSubmitButton}
          handleSubmit={(e) => rest.onEnterKeyPress?.(e)}
          isCommentEmpty={isEmpty}
          editorRef={editorRef}
        />
      )}
    </div>
  );
});

LiteTextEditor.displayName = "LiteTextEditor";
