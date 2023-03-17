import { create } from "zustand";

export interface IExploreStore {
  filter: string;
}

export const useExploreStore = create<IExploreStore>(() => ({
  filter: "",
}));
