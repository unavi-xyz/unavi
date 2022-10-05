import {
  PublicationContentWarning,
  PublicationMainFocus,
  PublicationMetadataDisplayTypes,
} from "../generated/graphql";
import { AppId } from "./types";

type Url = string;
type Markdown = string;
type Locale = string;
type MimeType = string;

export interface PublicationMetadataMedia {
  item: Url;
  /**
   * This is the mime type of media
   */
  type?: MimeType | null;

  /**
   * The alt tags for accessibility
   */
  altTag?: string | null;

  /**
   * The cover for any video or audio you attached
   */
  cover?: string | null;
}

export enum PublicationMetadataVersions {
  one = "1.0.0",
  // please use metadata v2 when doing anything! v1 is supported but discontinued.
  two = "2.0.0",
}

export enum PublicationMetadataDisplayType {
  number = "number",
  string = "string",
  date = "date",
}

interface PublicationMetadataAttribute {
  displayType?: PublicationMetadataDisplayTypes | undefined | null;
  traitType?: string | undefined | null;
  value: string;
}

export interface PublicationMetadata {
  /**
   * The metadata version.
   */
  version: PublicationMetadataVersions;

  /**
   * The metadata lens_id can be anything but if your uploading to ipfs
   * you will want it to be random.. using uuid could be an option!
   */
  metadata_id: string;

  /**
   * A human-readable description of the item.
   */
  description?: Markdown | undefined | null;

  /**
   * The content of a publication. If this is blank `media` must be defined or its out of spec.
   */
  content?: Markdown | undefined | null;

  /**
   * IOS 639-1 language code aka en or it and ISO 3166-1 alpha-2 region code aka US or IT aka en-US or it-IT
   * Full spec > https://tools.ietf.org/search/bcp47
   */
  locale: Locale;

  /**
   * Ability to tag your publication
   */
  tags?: string[] | undefined | null;

  /**
   * Ability to add a content warning
   */
  contentWarning?: PublicationContentWarning | undefined | null;

  /**
   * Main content focus that for this publication
   */
  mainContentFocus: PublicationMainFocus;

  /**
   * This is the URL that will appear below the asset's image on OpenSea and others etc
   * and will allow users to leave OpenSea and view the item on the site.
   */
  external_url?: Url | undefined | null;

  /**
   * Name of the item.
   */
  name: string;

  /**
   * These are the attributes for the item, which will show up on the OpenSea and others NFT trading websites on the
  item.
   */
  attributes: PublicationMetadataAttribute[];

  /**
   * legacy to support OpenSea will store any NFT image here.
   */
  image?: Url | undefined | null;

  /**
   * This is the mime type of image. This is used if you uploading more advanced cover images
   * as sometimes IPFS does not emit the content header so this solves the pr
   */
  imageMimeType?: MimeType | undefined | null;

  /**
   * This is lens supported attached media items to the publication
   */
  media?: PublicationMetadataMedia[] | undefined | null;

  /**
   * In spec for OpenSea and other providers - also used when using EMBED main publication focus
   * A URL to a multi-media attachment for the item. The file extensions GLTF, GLB, WEBM, MP4, M4V, OGV,
   * and OGG are supported, along with the audio-only extensions MP3, WAV, and OGA.
   * Animation_url also supports HTML pages, allowing you to build rich experiences and interactive NFTs using JavaScript canvas,
   * WebGL, and more. Scripts and relative paths within the HTML page are now supported. However, access to browser extensions is not supported.

   */
  animation_url?: Url | undefined | null;

  /**
   * This is the appId the content belongs to
   */
  appId?: AppId | undefined | null;
}
