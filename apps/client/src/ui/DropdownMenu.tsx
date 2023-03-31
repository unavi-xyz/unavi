"use client";

import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import React from "react";

type Props = DropdownPrimitive.DropdownMenuProps;

export const DropdownContent = React.forwardRef<HTMLDivElement, Props>(
  ({ children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.DropdownMenuPortal>
        <DropdownPrimitive.Content
          ref={ref}
          sideOffset={4}
          onCloseAutoFocus={(event) => event.preventDefault()}
          className="animate-scaleIn rounded-xl bg-neutral-50 shadow-md"
          {...rest}
        >
          {children}
        </DropdownPrimitive.Content>
      </DropdownPrimitive.DropdownMenuPortal>
    );
  }
);

DropdownContent.displayName = "DropdownContent";

export const DropdownMenu = DropdownPrimitive.Root;
export const DropdownTrigger = DropdownPrimitive.Trigger;
export const DropdownItem = DropdownPrimitive.Item;
export const DropdownSub = DropdownPrimitive.Sub;
export const DropdownSubTrigger = DropdownPrimitive.SubTrigger;

export const DropdownSubContent = React.forwardRef<HTMLDivElement, Props>(
  ({ children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.SubContent
        ref={ref}
        sideOffset={4}
        className="rounded-xl bg-neutral-50 shadow-md"
        {...rest}
      >
        {children}
      </DropdownPrimitive.SubContent>
    );
  }
);

DropdownSubContent.displayName = "DropdownSubContent";

export type DropdownMenuItemProps = DropdownPrimitive.DropdownMenuItemProps;
