/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { CircleArrowUp, CornerDownRight, RefreshCcw, Sparkles } from "lucide-react";
// ui
import { Tooltip } from "@plane/propel/tooltip";
// components
import { cn } from "@plane/utils";
import { RichTextEditor } from "@/components/editor/rich-text";
// hooks
import { useWorkspace } from "@/hooks/store/use-workspace";

type Props = {
  handleInsertText: (insertOnNextLine: boolean) => void;
  handleRegenerate: () => Promise<void>;
  isLoading: boolean;
  isRegenerating: boolean;
  onAskSubmit: (query: string) => Promise<void>;
  response: string | undefined;
  workspaceSlug: string;
};

export function AskPiMenu(props: Props) {
  const { handleInsertText, handleRegenerate, isLoading, isRegenerating, onAskSubmit, response, workspaceSlug } = props;
  // states
  const [query, setQuery] = useState("");
  // store hooks
  const { getWorkspaceBySlug } = useWorkspace();
  // derived values
  const workspaceId = getWorkspaceBySlug(workspaceSlug)?.id ?? "";

  const handleSubmit = async () => {
    const trimmed = query.trim();
    if (!trimmed || isLoading) return;
    await onAskSubmit(trimmed);
    setQuery("");
  };

  return (
    <>
      <div
        className={cn("flex items-center gap-3 px-4 py-3.5", {
          "items-start": response,
        })}
      >
        <span className="grid size-7 flex-shrink-0 place-items-center rounded-full border border-subtle text-secondary">
          <Sparkles className="size-3" />
        </span>
        {response ? (
          <div>
            <RichTextEditor
              editable={false}
              displayConfig={{
                fontSize: "small-font",
              }}
              id="editor-ai-response"
              initialValue={response}
              containerClassName="!p-0 border-none"
              editorClassName="!pl-0"
              workspaceId={workspaceId}
              workspaceSlug={workspaceSlug}
            />
            <div className="mt-3 flex items-center gap-4">
              <button
                type="button"
                className="rounded-sm p-1 text-13 font-medium text-tertiary outline-none hover:bg-layer-1"
                onClick={() => handleInsertText(false)}
              >
                Replace selection
              </button>
              <Tooltip tooltipContent="Add to next line">
                <button
                  type="button"
                  className="grid size-6 flex-shrink-0 place-items-center rounded-sm outline-none hover:bg-layer-1"
                  onClick={() => handleInsertText(true)}
                >
                  <CornerDownRight className="size-4 text-tertiary" />
                </button>
              </Tooltip>
              <Tooltip tooltipContent="Re-generate response">
                <button
                  type="button"
                  className="grid size-6 flex-shrink-0 place-items-center rounded-sm outline-none hover:bg-layer-1"
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    handleRegenerate();
                  }}
                  disabled={isRegenerating}
                >
                  <RefreshCcw
                    className={cn("size-4 text-tertiary", {
                      "animate-spin": isRegenerating,
                    })}
                  />
                </button>
              </Tooltip>
            </div>
          </div>
        ) : isLoading ? (
          <p className="text-13 text-secondary">Pi is generating response...</p>
        ) : (
          <p className="text-13 text-secondary">Ask Pi anything about the selected text.</p>
        )}
      </div>
      <div className="px-4 py-3">
        <div className="flex items-center gap-2 rounded-md border border-subtle p-2">
          <span className="grid size-3 flex-shrink-0 place-items-center">
            <Sparkles className="size-3 text-secondary" />
          </span>
          <input
            type="text"
            className="w-full border-none bg-transparent text-13 outline-none placeholder:text-placeholder"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSubmit();
              }
            }}
            placeholder="Tell AI what to do..."
            disabled={isLoading}
          />
          <button
            type="button"
            className={cn("grid size-4 flex-shrink-0 place-items-center transition-opacity", {
              "opacity-40 cursor-not-allowed": !query.trim() || isLoading,
            })}
            onClick={handleSubmit}
            disabled={!query.trim() || isLoading}
            aria-label="Submit query to Pi"
          >
            <CircleArrowUp
              className={cn("size-4 text-secondary", {
                "text-accent-primary": query.trim() && !isLoading,
              })}
            />
          </button>
        </div>
      </div>
    </>
  );
}
