export type Message = {
  id: string;
  username: string;
  text: string;
  time: number;
};

export type Identity = {
  isGuest: boolean;
  did: string;
};
