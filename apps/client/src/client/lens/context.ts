import { createContext } from "react";
import { Client, createClient } from "urql";

import { API_URL } from "./constants";

export interface ILensContext {
  client: Client;
  handle: string | undefined;
  switchProfile: (handle: string | undefined) => void;
  setAccessToken: (accessToken: string) => void;
}

const defaultClient = createClient({ url: API_URL });

export const initialContext: ILensContext = {
  client: defaultClient,
  handle: undefined,
  switchProfile: () => {},
  setAccessToken: () => {},
};

export const LensContext = createContext<ILensContext>(initialContext);
