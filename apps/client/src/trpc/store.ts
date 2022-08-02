import create from "zustand";

export const useJWTStore = create((set) => ({
  token: "",
}));
