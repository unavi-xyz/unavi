import { useContext } from "react";
import { createContext } from "react";

import { Materials } from "../types";

interface MaterialContext {
  materials: Materials;
}

export const MaterialContext = createContext<MaterialContext>({
  materials: {},
});

interface MaterialProviderProps {
  children: React.ReactNode;
  materials: Materials;
}

export function MaterialProvider({
  materials,
  children,
}: MaterialProviderProps) {
  return (
    <MaterialContext.Provider value={{ materials }}>
      {children}
    </MaterialContext.Provider>
  );
}

export function useMaterial(id: string | undefined) {
  const { materials } = useContext(MaterialContext);
  if (!id) return;
  return materials[id];
}
