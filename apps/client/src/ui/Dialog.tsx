"use client";

import * as DialogPrimitive from "@radix-ui/react-dialog";
import React from "react";

interface Props extends DialogPrimitive.DialogContentProps {
  open?: boolean;
  title?: string;
  description?: string;
  autoFocus?: boolean;
}

const DialogContent = React.forwardRef<HTMLDivElement, Props>(
  ({ open = true, autoFocus = true, title, description, children, ...rest }, ref) => {
    return (
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay
          className={`fixed inset-0 z-40 h-full w-full bg-neutral-900/40 backdrop-blur-sm ${
            open ? "animate-fadeIn" : "animate-fadeOut"
          }`}
        />

        <DialogPrimitive.Content
          ref={ref}
          onOpenAutoFocus={autoFocus ? undefined : (e) => e.preventDefault()}
          className={`fixed inset-0 z-50 m-auto h-fit w-full max-w-xl rounded-2xl bg-white py-8 px-10 shadow-md ${
            open ? "animate-scaleIn" : "animate-scaleOut"
          }`}
          {...rest}
        >
          <DialogPrimitive.Title className="text-center text-3xl font-bold">
            {title}
          </DialogPrimitive.Title>

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
