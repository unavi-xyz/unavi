import { createClient } from "urql";

import { API_URL } from "@wired-xr/lens";

export const lensClient = createClient({
  url: API_URL,
});
