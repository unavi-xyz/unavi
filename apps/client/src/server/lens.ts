import { createClient } from "urql";

import { API_URL } from "../lib/lens/constants";

export const lensClient = createClient({ url: API_URL });
