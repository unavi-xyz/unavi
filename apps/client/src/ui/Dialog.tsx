"use client";

import * as DialogPrimitive from "@radix-ui/react-dialog";
import React from "react";

interface Props extends DialogPrimitive.DialogContentProps {
  title?: string;
  description?: string;
  autoFocus?: boolean;
  size?: "normal" | "large";
}

const DialogContent = React.forwardRef<HTMLDivElement, Props>(
  ({ autoFocus = true, title, description, size = "normal", children, ...rest }, ref) => {
    return (
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-40 h-full w-full animate-fadeIn bg-neutral-900/40 backdrop-blur-sm" />

        <DialogPrimitive.Content
          ref={ref}
          onOpenAutoFocus={autoFocus ? undefined : (e) => e.preventDefault()}
          className={`fixed inset-0 z-50 m-auto h-fit max-h-screen w-full animate-scaleIn rounded-3xl bg-white p-8 shadow-md ${
            size === "normal" ? "max-w-lg" : "max-w-4xl"
          }`}
          {...rest}
        >
          {title && (
            <DialogPrimitive.Title className="text-center text-2xl font-black">
              {title}
            </DialogPrimitive.Title>
          )}

          {description && (
            <DialogPrimitive.Description className="pt-2 text-center text-lg text-neutral-500">
              {description}
            </DialogPrimitive.Description>
          )}

          <div className="pt-4">{children}</div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    );
  }
);

DialogContent.displayName = "Dialog";

export default DialogContent;
export const DialogRoot = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
