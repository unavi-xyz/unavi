import { createContext } from "react";

import { Assets } from "../../types";

interface AssetContext {
  assets: Assets;
}

export const AssetContext = createContext<AssetContext>({
  assets: {},
});

interface AssetProviderProps {
  children: React.ReactNode;
  assets: Assets;
}

export function AssetProvider({ assets, children }: AssetProviderProps) {
  return <AssetContext.Provider value={{ assets }}>{children}</AssetContext.Provider>;
}
