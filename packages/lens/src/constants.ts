export enum SessionStorage {
  AutoLogin = "auto_login",
  ActiveHomeToken = "active_home_token",
}

export enum LocalStorage {
  PreviousHandle = "prev_handle",

  HomeToken = "home_token",

  AccessToken = "access_token",
  AccessExpire = "access_expire",
  RefreshToken = "refresh_token",
  RefreshExpire = "refresh_expire",
}

export enum ContractAddress {
  Zero = "0x0000000000000000000000000000000000000000",
  LensPeriphery = "0xD5037d72877808cdE7F669563e9389930AF404E8",
  LensHub = "0x60Ae865ee4C725cd04353b5AAb364553f56ceF82",
  EmptyCollectModule = "0x0BE6bD7092ee83D44a6eC1D949626FeE48caB30c",
}

export const API_URL = "https://api-mumbai.lens.dev/";
export const HIDDEN_MESSAGE = "This publication has been hidden";
export const HANDLE_ENDING = ".test";
