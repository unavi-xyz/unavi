import { createContext, useContext } from "react";

import {
  AuthContext,
  AuthProvider,
  IAuthContext,
  initialAuthContext,
} from "./AuthProvider";
import {
  ClientContext,
  ClientProvider,
  IClientContext,
  initialClientContext,
} from "./ClientProvider";

type ILensContext = IClientContext & IAuthContext;

export const LensContext = createContext<ILensContext>({
  ...initialAuthContext,
  ...initialClientContext,
});

export function LensProvider({ children }: { children: React.ReactNode }) {
  return (
    <ClientProvider>
      <AuthProvider>
        <ProviderCombiner>{children}</ProviderCombiner>
      </AuthProvider>
    </ClientProvider>
  );
}

function ProviderCombiner({ children }: { children: React.ReactNode }) {
  const clientContext = useContext(ClientContext);
  const authContext = useContext(AuthContext);

  return (
    <LensContext.Provider
      value={{
        ...clientContext,
        ...authContext,
      }}
    >
      {children}
    </LensContext.Provider>
  );
}
