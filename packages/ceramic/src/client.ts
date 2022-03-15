import CeramicClient from "@ceramicnetwork/http-client";
import ThreeIdResolver from "@ceramicnetwork/3id-did-resolver";
import { TileLoader } from "@glazed/tile-loader";

import { API_URL } from "./constants";

//authenticate the user using "ceramic"
//keep "ceramicRead" unauthenticated, so you can load data when not logged in
export const ceramic = new CeramicClient(API_URL) as any;
export const ceramicRead = new CeramicClient(API_URL) as any;

export const resolver = ThreeIdResolver.getResolver(ceramic);
export const loader = new TileLoader({ ceramic, cache: true });
