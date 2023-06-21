"use client";

import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import { forwardRef } from "react";

type Props = DropdownPrimitive.DropdownMenuContentProps;

export const DropdownContent = forwardRef<HTMLDivElement, Props>(
  ({ children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.DropdownMenuPortal>
        <DropdownPrimitive.Content
          ref={ref}
          sideOffset={4}
          onCloseAutoFocus={(event) => event.preventDefault()}
          className="animate-scaleIn shadow-dark z-50 mx-4 rounded-xl bg-white"
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

export const DropdownSubContent = forwardRef<HTMLDivElement, Props>(
  ({ children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.SubContent
        ref={ref}
        sideOffset={4}
        className="shadow-dark rounded-xl bg-white"
        {...rest}
      >
        {children}
      </DropdownPrimitive.SubContent>
    );
  }
);

DropdownSubContent.displayName = "DropdownSubContent";

export type DropdownMenuItemProps = DropdownPrimitive.DropdownMenuItemProps;
