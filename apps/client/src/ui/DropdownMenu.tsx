import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import React from "react";

interface Props extends DropdownPrimitive.DropdownMenuProps {
  open: boolean;
}

export const DropdownContent = React.forwardRef<HTMLDivElement, Props>(
  ({ open, children, ...rest }, ref) => {
    return (
      <DropdownPrimitive.Content
        ref={ref}
        sideOffset={8}
        className={`z-40 rounded-lg bg-white shadow ${
          open ? "animate-scaleIn" : "animate-scaleOut"
        }`}
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
