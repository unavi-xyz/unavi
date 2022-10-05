export enum ProfileMetadataDisplayType {
  number = "number",
  string = "string",
  date = "date",
}

export enum ProfileMetadataVersions {
  one = "1.0.0",
}

export interface AttributeData {
  displayType?: ProfileMetadataDisplayType;
  traitType?: string;
  value: string;
  key: string;
}

export interface ProfileMetadata {
  /**
   * The metadata version.
   */
  version: ProfileMetadataVersions;

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
