import { NANOID_LENGTH, NANOID_SHORT_LENGTH } from "../nanoid";

export const AUTH_USER_TABLE_NAME = "auth_user";
export const AUTH_KEY_TABLE_NAME = "auth_key";
export const AUTH_SESSION_TABLE_NAME = "auth_session";

export const WORLD_ID_LENGTH = NANOID_SHORT_LENGTH;
export const WORLD_TITLE_LENGTH = 80;
export const WORLD_DESCRIPTION_LENGTH = 1200;
export const WORLD_HOST_LENGTH = 255;

export const FILE_KEY_LENGTH = NANOID_SHORT_LENGTH;

export const MIN_USERNAME_LENGTH = 3;
export const MAX_USERNAME_LENGTH = 15;
export const MAX_PROFILE_BIO_LENGTH = 160;

export const USER_ID_LENGTH = 15;
export const DID_ID_LENGTH = 15;

export const ETH_ADDRESS_LENGTH = 42;
export const ETH_AUTH_ID_LENGTH = NANOID_LENGTH;
export const ETH_AUTH_NONCE_LENGTH = 96;
