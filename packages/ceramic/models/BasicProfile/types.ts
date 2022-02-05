export type IPFSUrl = string;
export type PositiveInteger = number;

export interface BasicProfile {
  name?: string;
  image?: ImageSources;
  description?: string;
  emoji?: string;
  background?: ImageSources;
  birthDate?: string;
  url?: string;
  gender?: string;
  homeLocation?: string;
  residenceCountry?: string;
  nationalities?: [string, ...string[]];
  affiliations?: string[];
  [k: string]: unknown;
}
export interface ImageSources {
  original: ImageMetadata;
  alternatives?: ImageMetadata[];
  [k: string]: unknown;
}
export interface ImageMetadata {
  src: IPFSUrl;
  mimeType: string;
  width: PositiveInteger;
  height: PositiveInteger;
  size?: PositiveInteger;
  [k: string]: unknown;
}
