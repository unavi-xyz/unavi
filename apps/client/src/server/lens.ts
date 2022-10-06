import { createClient } from "urql";

import { API_URL } from "../client/lens/constants";

export const lensClient = createClient({ url: API_URL });
