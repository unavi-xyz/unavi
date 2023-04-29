/// <reference types="lucia-auth" />
declare namespace Lucia {
  type Auth = import("./src/server/auth/lucia").Auth;
  type UserAttributes = {
    username: string;
    address?: string;
  };
}
