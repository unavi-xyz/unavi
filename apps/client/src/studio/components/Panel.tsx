"use client";

import React from "react";
import {
  Panel as RpPanel,
  PanelGroup as RpPanelGroup,
  PanelProps,
  PanelResizeHandle as RpPanelResizeHandle,
} from "react-resizable-panels";

import { useStudio } from "./Studio";

interface Props extends PanelProps {
  children: React.ReactNode;
}

/**
 * Wrapper around react-resizable-panels that manages the onResize callback.
 * Also marks these components as client side.
 */
export const Panel = React.forwardRef<any, Props>(({ children, ...rest }, ref) => {
  const { resize } = useStudio();

  return (
    <RpPanel ref={ref} onResize={resize} {...rest}>
      {children}
    </RpPanel>
  );
});

Panel.displayName = "Panel";

export const PanelGroup = RpPanelGroup;
export const PanelResizeHandle = RpPanelResizeHandle;
