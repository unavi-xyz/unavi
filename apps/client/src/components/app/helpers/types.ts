import { Triplet } from "@react-three/cannon";

export type YUser = {
  id: string;
  messages: YMessage[];
  location: {
    position: Triplet;
    rotation: number;
  };
};

export const emptyUser: YUser = {
  id: "",
  messages: [],
  location: {
    position: [0, 0, 0],
    rotation: 0,
  },
};

export type YMessage = {
  text: string;
  time: number;
};

export type YLocation = {
  position: Triplet;
  rotation: number;
};

export type Message = {
  id: string;
  username: string;
  text: string;
  time: number;
};
