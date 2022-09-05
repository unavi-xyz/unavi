import { createClient } from "urql";

import { API_URL } from "./constants";

export const lensClient = createClient({ url: API_URL });
