import CeramicClient from "@ceramicnetwork/http-client";
import ThreeIdResolver from "@ceramicnetwork/3id-did-resolver";
import { TileLoader } from "@glazed/tile-loader";

import { API_URL } from "./constants";

export const ceramicRead = new CeramicClient(API_URL);
export const ceramic = new CeramicClient(API_URL);
export const resolver = ThreeIdResolver.getResolver(ceramic);
export const loader = new TileLoader({ ceramic, cache: true });
