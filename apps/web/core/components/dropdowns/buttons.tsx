/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import type { MouseEventHandler, Ref } from "react";
// helpers
import { Button } from "@plane/propel/button";
import { Tooltip } from "@plane/propel/tooltip";
import { cn } from "@plane/utils";
// types
import { usePlatformOS } from "@/hooks/use-platform-os";
import { BACKGROUND_BUTTON_VARIANTS, BORDER_BUTTON_VARIANTS } from "./constants";
import type { TButtonVariants } from "./types";

export type DropdownButtonProps = {
  as?: "button" | "div";
  buttonRef?: Ref<HTMLButtonElement>;
  children: React.ReactNode;
  className?: string;
  containerClassName?: string;
  disabled?: boolean;
  isActive: boolean;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  tabIndex?: number;
  tooltipContent?: string | React.ReactNode | null;
  tooltipHeading: string;
  showTooltip: boolean;
  variant: TButtonVariants;
  renderToolTipByDefault?: boolean;
};

type ButtonProps = {
  as?: "button" | "div";
  buttonRef?: Ref<HTMLButtonElement>;
  children: React.ReactNode;
  className?: string;
  containerClassName?: string;
  disabled?: boolean;
  isActive: boolean;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  tabIndex?: number;
  tooltipContent?: string | React.ReactNode | null;
  tooltipHeading: string;
  showTooltip: boolean;
  renderToolTipByDefault?: boolean;
};

export function DropdownButton(props: DropdownButtonProps) {
  const {
    as = "button",
    buttonRef,
    children,
    className,
    containerClassName,
    disabled = false,
    isActive,
    onClick,
    tooltipContent,
    renderToolTipByDefault = true,
    tabIndex,
    tooltipHeading,
    showTooltip,
    variant,
  } = props;
  const ButtonToRender: React.FC<ButtonProps> = BORDER_BUTTON_VARIANTS.includes(variant)
    ? BorderButton
    : BACKGROUND_BUTTON_VARIANTS.includes(variant)
      ? BackgroundButton
      : TransparentButton;

  return (
    <ButtonToRender
      as={as}
      buttonRef={buttonRef}
      className={className}
      containerClassName={containerClassName}
      disabled={disabled}
      isActive={isActive}
      onClick={onClick}
      tabIndex={tabIndex}
      tooltipContent={tooltipContent}
      tooltipHeading={tooltipHeading}
      showTooltip={showTooltip}
      renderToolTipByDefault={renderToolTipByDefault}
    >
      {children}
    </ButtonToRender>
  );
}

function BorderButton(props: ButtonProps) {
  const {
    as = "button",
    buttonRef,
    children,
    className,
    containerClassName,
    disabled = false,
    isActive,
    onClick,
    tooltipContent,
    renderToolTipByDefault,
    tabIndex,
    tooltipHeading,
    showTooltip,
  } = props;
  const { isMobile } = usePlatformOS();
  const content = (
    <div
      className={cn(
        "flex h-full w-full items-center justify-start gap-1.5 border-[0.5px] border-strong",
        {
          "bg-layer-transparent-active": isActive,
        },
        className
      )}
    >
      {children}
    </div>
  );

  return (
    <Tooltip
      tooltipHeading={tooltipHeading}
      tooltipContent={<>{tooltipContent}</>}
      disabled={!showTooltip}
      isMobile={isMobile}
      renderByDefault={renderToolTipByDefault}
    >
      {as === "button" ? (
        <Button
          ref={buttonRef}
          variant="ghost"
          size="sm"
          onClick={onClick}
          disabled={disabled}
          tabIndex={tabIndex}
          className={cn(
            "clickable block h-full max-w-full items-center justify-start gap-1.5 border-[0.5px] border-strong outline-none",
            {
              "cursor-not-allowed text-secondary": disabled,
              "cursor-pointer": !disabled,
            },
            containerClassName,
            {
              "bg-layer-transparent-active": isActive,
            },
            className
          )}
        >
          {children}
        </Button>
      ) : (
        content
      )}
    </Tooltip>
  );
}

function BackgroundButton(props: ButtonProps) {
  const {
    as = "button",
    buttonRef,
    children,
    className,
    containerClassName,
    disabled = false,
    onClick,
    tabIndex,
    tooltipContent,
    tooltipHeading,
    renderToolTipByDefault,
    showTooltip,
  } = props;
  const { isMobile } = usePlatformOS();
  const content = (
    <div
      className={cn(
        "flex h-full w-full items-center justify-between gap-1.5 bg-layer-3 hover:bg-layer-1-hover",
        className
      )}
    >
      {children}
    </div>
  );
  return (
    <Tooltip
      tooltipHeading={tooltipHeading}
      tooltipContent={<>{tooltipContent}</>}
      disabled={!showTooltip}
      isMobile={isMobile}
      renderByDefault={renderToolTipByDefault}
    >
      {as === "button" ? (
        <Button
          ref={buttonRef}
          variant="ghost"
          size="sm"
          onClick={onClick}
          disabled={disabled}
          tabIndex={tabIndex}
          className={cn(
            "clickable block h-full max-w-full items-center justify-between gap-1.5 bg-layer-3 outline-none hover:bg-layer-1-hover",
            {
              "cursor-not-allowed text-secondary": disabled,
              "cursor-pointer": !disabled,
            },
            containerClassName,
            className
          )}
        >
          {children}
        </Button>
      ) : (
        content
      )}
    </Tooltip>
  );
}

function TransparentButton(props: ButtonProps) {
  const {
    as = "button",
    buttonRef,
    children,
    className,
    containerClassName,
    disabled = false,
    isActive,
    onClick,
    tabIndex,
    tooltipContent,
    tooltipHeading,
    renderToolTipByDefault,
    showTooltip,
  } = props;
  const { isMobile } = usePlatformOS();
  const content = (
    <div
      className={cn(
        "flex h-full w-full items-center justify-between gap-1.5",
        {
          "bg-layer-transparent-active": isActive,
        },
        className
      )}
    >
      {children}
    </div>
  );
  return (
    <Tooltip
      tooltipHeading={tooltipHeading}
      tooltipContent={<>{tooltipContent}</>}
      disabled={!showTooltip}
      isMobile={isMobile}
      renderByDefault={renderToolTipByDefault}
    >
      {as === "button" ? (
        <Button
          ref={buttonRef}
          variant="ghost"
          size="sm"
          onClick={onClick}
          disabled={disabled}
          tabIndex={tabIndex}
          className={cn(
            "clickable block h-full max-w-full items-center justify-between gap-1.5 outline-none",
            {
              "cursor-not-allowed text-secondary": disabled,
              "cursor-pointer": !disabled,
            },
            containerClassName,
            {
              "bg-layer-transparent-active": isActive,
            },
            className
          )}
        >
          {children}
        </Button>
      ) : (
        content
      )}
    </Tooltip>
  );
}
