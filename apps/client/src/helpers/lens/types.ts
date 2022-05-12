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
  version: "1.0.0";

  /**
   * The metadata id can be anything but if your uploading to ipfs
   * you will want it to be random.. using uuid could be an option!
   */
  metadata_id: string;

  /**
   * The display name for the profile
   */
  name?: string;

  /**
   * The bio for the profile
   */
  bio?: string;

  /**
   * Cover picture
   */
  cover_picture?: string;

  /**
   * Any custom attributes can be added here to save state for a profile
   */
  attributes: AttributeData[];
}
