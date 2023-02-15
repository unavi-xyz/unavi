import * as DialogPrimitive from "@radix-ui/react-dialog";
import React from "react";

interface Props extends DialogPrimitive.DialogProps {
  open: boolean;
  title?: string;
  description?: string;
}

const Dialog = React.forwardRef<HTMLDivElement, Props>(
  ({ open, title, description, children, ...rest }, ref) => {
    return (
      <DialogPrimitive.Root open={open} {...rest}>
        <DialogPrimitive.Portal>
          <DialogPrimitive.Overlay
            className={`fixed inset-0 z-40 h-full w-full bg-neutral-900/40 backdrop-blur-sm ${
              open ? "animate-fadeIn" : "animate-fadeOut"
            }`}
          />

          <DialogPrimitive.Content
            ref={ref}
            className={`fixed inset-0 z-50 m-auto h-fit w-full max-w-xl rounded-2xl bg-white py-8 px-10 shadow-md ${
              open ? "animate-scaleIn" : "animate-scaleOut"
            }`}
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
      </DialogPrimitive.Root>
    );
  }
);

Dialog.displayName = "Dialog";

export default Dialog;
