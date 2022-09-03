type Url = string;
type Markdown = string;
type MimeType = string;

export enum AppId {
  Space = "wired/space",
  Avatar = "wired/avatar",
}

export enum MetadataVersions {
  one = "1.0.0",
}

export enum MetadataDisplayType {
  number = "number",
  string = "string",
  date = "date",
}

export interface AttributeData {
  displayType?: MetadataDisplayType;
  traitType?: string;
  value: string;
  key: string;
}

export interface ProfileMetadata {
  /**
   * The metadata version.
   */
  version: MetadataVersions;

  /**
   * The metadata id can be anything but if your uploading to ipfs
   * you will want it to be random.. using uuid could be an option!
   */
  metadata_id: string;

  /**
   * The display name for the profile
   */
  name: string | null;

  /**
   * The bio for the profile
   */
  bio: string | null;

  /**
   * Cover picture
   */
  cover_picture: string | null;

  /**
   * Any custom attributes can be added here to save state for a profile
   */
  attributes: AttributeData[];
}

interface MetadataMedia {
  item: Url;
  /**
   * This is the mime type of media
   */
  type: MimeType;
}

interface MetadataAttribute {
  displayType?: MetadataDisplayType;
  traitType?: string;
  value: string;
}

export interface Metadata {
  /**
   * The metadata version.
   */
  version: MetadataVersions;

  /**
   * The metadata lens_id can be anything but if your uploading to ipfs
   * you will want it to be random.. using uuid could be an option!
   */
  metadata_id: string;

  /**
   * A human-readable description of the item.
   */
  description?: Markdown;

  /**
   * The content of a publication. If this is blank `media` must be defined or its out of spec.
   */
  content?: Markdown;

  /**
   * This is the URL that will appear below the asset's image on OpenSea and others etc
   * and will allow users to leave OpenSea and view the item on the site.
   */
  external_url?: Url;

  /**
   * Name of the item.
   */
  name: string;

  /**
   * These are the attributes for the item, which will show up on the OpenSea and others NFT trading websites on the
  item.
   */
  attributes: MetadataAttribute[];

  /**
   * legacy to support OpenSea will store any NFT image here.
   */
  image?: Url;

  /**
   * This is the mime type of image. This is used if you uploading more advanced cover images
   * as sometimes IPFS does not emit the content header so this solves the pr
   */
  imageMimeType?: MimeType;

  /**
   * This is lens supported attached media items to the publication
   */
  media?: MetadataMedia[];

  /**
   * Legacy for OpenSea and other providers
   * A URL to a multi-media attachment for the item. The file extensions GLTF, GLB, WEBM, MP4, M4V, OGV,
   * and OGG are supported, along with the audio-only extensions MP3, WAV, and OGA.
   * Animation_url also supports HTML pages, allowing you to build rich experiences and interactive NFTs using JavaScript canvas,
   * WebGL, and more. Scripts and relative paths within the HTML page are now supported. However, access to browser extensions is not supported.

   */
  animation_url?: Url;

  /**
   * This is the appId the content belongs to
   */
  appId?: AppId;
}
