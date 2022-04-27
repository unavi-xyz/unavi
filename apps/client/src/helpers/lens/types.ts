type Url = string;
type MetadataVersions = string;

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
  name: string;

  /**
   * The bio for the profile
   */
  bio: string;

  /**
   * The location
   */
  location: string;

  /**
   * Cover picture
   */
  cover_picture: Url;

  /**
   * social fields right now we only support `website` and `twitter` right now the keys must match these
   * names and it extract that out in the profile schema for you
   */
  social: AttributeData[];

  /**
   * Any custom attributes can be added here to save state for a profile
   */
  attributes: AttributeData[];
}
