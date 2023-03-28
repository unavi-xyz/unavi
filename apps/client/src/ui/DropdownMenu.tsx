"use client";

import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import React from "react";

type Props = DropdownPrimitive.DropdownMenuProps;

export const DropdownContent = React.forwardRef<HTMLDivElement, Props>(
  ({ children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.Content
        ref={ref}
        sideOffset={4}
        onCloseAutoFocus={(event) => event.preventDefault()}
        className="z-40 animate-scaleIn rounded-xl bg-white shadow-md"
        {...rest}
      >
        {children}
      </DropdownPrimitive.Content>
    );
  }
);

DropdownContent.displayName = "DropdownContent";

export const DropdownMenu = DropdownPrimitive.Root;
export const DropdownTrigger = DropdownPrimitive.Trigger;
export const DropdownItem = DropdownPrimitive.Item;

export type DropdownMenuItemProps = DropdownPrimitive.DropdownMenuItemProps;
