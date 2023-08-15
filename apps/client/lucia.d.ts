/* eslint-disable @typescript-eslint/ban-types */
/// <reference types="lucia" />
declare namespace Lucia {
  type Auth = import("./src/server/auth/lucia").Auth;
  type DatabaseUserAttributes = {
    username: string;
    did: string;
    did_id: string;
    address?: string;
  };
  type DatabaseSessionAttributes = {};
}
