import { create } from "zustand";

export interface IClientStore {
  worldUri: string;
}

export const useClientStore = create<IClientStore>((set, get) => ({
  worldUri: "",
}));
