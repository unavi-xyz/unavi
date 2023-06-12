"use client";

import { createContext, Dispatch, SetStateAction, useContext, useMemo, useState } from "react";

export type TreeContextType = {
  draggingId: string | null;
  openIds: string[];
  setDraggingId: Dispatch<SetStateAction<string | null>>;
  setOpenIds: Dispatch<SetStateAction<string[]>>;
  setTreeIds: Dispatch<SetStateAction<string[]>>;
  treeIds: string[];
};

const defaultContext: TreeContextType = {
  draggingId: null,
  openIds: [],
  setDraggingId: () => {},
  setOpenIds: () => {},
  setTreeIds: () => {},
  treeIds: [],
};

const TreeContext = createContext<TreeContextType>(defaultContext);

export function useTree() {
  return useContext(TreeContext);
}

interface Props {
  children: React.ReactNode;
}

export default function Tree({ children }: Props) {
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [openIds, setOpenIds] = useState<string[]>([]);
  const [treeIds, setTreeIds] = useState<string[]>([]);

  const value = useMemo(() => {
    return {
      draggingId,
      openIds,
      setDraggingId,
      setOpenIds,
      setTreeIds,
      treeIds,
    };
  }, [draggingId, openIds, treeIds]);

  return <TreeContext.Provider value={value}>{children}</TreeContext.Provider>;
}
