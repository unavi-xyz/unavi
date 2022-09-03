import { createClient } from "urql";

import { API_URL } from "@wired-labs/lens";

export const lensClient = createClient({
  url: API_URL,
});
