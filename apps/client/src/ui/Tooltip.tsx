"use client";

import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import React, { useEffect, useState } from "react";

interface Props {
  text: string;
  delayDuration?: number;
  side?: "left" | "right" | "bottom" | "top";
  children: React.ReactNode;
}

const Tooltip = React.forwardRef<HTMLDivElement, Props>(
  ({ text, delayDuration = 400, side = "bottom", children }, ref) => {
    const [open, setOpen] = useState(false);
    const [visible, setVisible] = useState(false);

    // Closing animation
    useEffect(() => {
      if (!visible) {
        const timer = setTimeout(() => setOpen(false), 200);
        return () => clearTimeout(timer);
      }
    }, [visible]);

    return (
      <TooltipPrimitive.Provider>
        <TooltipPrimitive.Root
          delayDuration={delayDuration}
          open={open}
          onOpenChange={(open) => {
            if (open) {
              setOpen(true);
              setVisible(true);
            } else {
              setVisible(open);
            }
          }}
        >
          <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>

          <TooltipPrimitive.Portal>
            <TooltipPrimitive.Content
              ref={ref}
              side={side}
              sideOffset={10}
              className={`z-50 rounded-lg bg-black/90 px-3 py-1.5 text-xs font-semibold text-white shadow-lg backdrop-blur transition ${
                visible ? "animate-scaleInFull" : "animate-scaleOutFull"
              }`}
            >
              {text}
            </TooltipPrimitive.Content>
          </TooltipPrimitive.Portal>
        </TooltipPrimitive.Root>
      </TooltipPrimitive.Provider>
    );
  }
);

Tooltip.displayName = "Tooltip";

export default Tooltip;
