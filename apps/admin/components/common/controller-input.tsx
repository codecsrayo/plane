/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React, { useState } from "react";
import type { Control, FieldValues } from "react-hook-form";
import { Controller } from "react-hook-form";
// icons
import { Eye, EyeOff } from "lucide-react";
// plane internal packages
import { Input, TextArea } from "@plane/ui";
import { cn } from "@plane/utils";

type Props<TFieldValues extends FieldValues = FieldValues> = {
  control: Control<TFieldValues>;
  type: "text" | "password";
  name: string;
  label: string;
  description?: string | React.ReactNode;
  placeholder: string;
  error: boolean;
  required: boolean;
  disabled?: boolean;
  rightContent?: React.ReactNode;
  multiline?: boolean;
  textAreaClassName?: string;
};

export type TControllerInputFormField = {
  key: string;
  type: "text" | "password";
  label: string;
  description?: string | React.ReactNode;
  placeholder: string;
  error: boolean;
  required: boolean;
  disabled?: boolean;
  rightContent?: React.ReactNode;
  multiline?: boolean;
  textAreaClassName?: string;
};

export function ControllerInput<TFieldValues extends FieldValues = FieldValues>(
  props: Props<TFieldValues>
) {
  const {
    name,
    control,
    type,
    label,
    description,
    placeholder,
    error,
    required,
    disabled = false,
    rightContent,
    multiline = false,
    textAreaClassName,
  } = props;
  // states
  const [showPassword, setShowPassword] = useState(false);

  return (
    <div className="flex flex-col gap-1">
      <h4 className="text-13 text-tertiary">{label}</h4>
      <div className="relative">
        <Controller
          control={control}
          name={name}
          rules={{ required: required ? `${label} is required.` : false }}
          render={({ field: { value, onChange, ref } }) =>
            multiline ? (
              <TextArea
                id={name}
                name={name}
                value={value}
                onChange={onChange}
                ref={ref}
                hasError={error}
                placeholder={placeholder}
                disabled={disabled}
                rows={8}
                className={cn("h-64 min-h-64 w-full resize-y rounded-md font-medium", textAreaClassName)}
              />
            ) : (
              <Input
                id={name}
                name={name}
                type={type === "password" && showPassword ? "text" : type}
                value={value}
                onChange={onChange}
                ref={ref}
                hasError={error}
                placeholder={placeholder}
                disabled={disabled}
                className={cn("w-full rounded-md font-medium", {
                  "pr-10": (type === "password" && !rightContent) || (type !== "password" && !!rightContent),
                  "pr-20": type === "password" && rightContent,
                })}
              />
            )
          }
        />
        {!multiline && (type === "password" || rightContent) && (
          <div className="absolute top-2.5 right-3 flex items-center gap-2 text-placeholder">
            {type === "password" &&
              (showPassword ? (
                <button
                  type="button"
                  tabIndex={-1}
                  className="flex items-center justify-center"
                  onClick={() => setShowPassword(false)}
                  disabled={disabled}
                >
                  <EyeOff className="h-4 w-4" />
                </button>
              ) : (
                <button
                  type="button"
                  tabIndex={-1}
                  className="flex items-center justify-center"
                  onClick={() => setShowPassword(true)}
                  disabled={disabled}
                >
                  <Eye className="h-4 w-4" />
                </button>
              ))}
            {rightContent}
          </div>
        )}
      </div>
      {description && <p className="pt-0.5 text-11 text-tertiary">{description}</p>}
    </div>
  );
}
