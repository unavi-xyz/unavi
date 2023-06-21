export type VRMMetadataDef = {
  name: string;
  version?: string;
  authors: string[];
  copyrightInformation?: string;
  contactInformation?: string;
  references?: string[];
  thirdPartyLicenses?: string;
  thumbnailImage?: number;
  licenseUrl: string;
  avatarPermission?: "onlyAuthor" | "onlySeparatelyLicensedPerson" | "everyone";
  allowExcessivelyViolentUsage?: boolean;
  allowExcessivelySexualUsage?: boolean;
  commercialUsage?: "personalNonProfit" | "personalProfit" | "corporation";
  allowPoliticalOrReligiousUsage?: boolean;
  allowAntisocialOrHateUsage?: boolean;
  creditNotation?: "required" | "unnecessary";
  allowRedistribution?: boolean;
  modification?:
    | "prohibited"
    | "allowModification"
    | "allowModificationRedistribution";
  otherLicenseUrl?: string;
};

export type VRMDef = {
  specVersion: "1.0";
  humanoid: unknown;
  meta: VRMMetadataDef;
  firstPerson: unknown;
  expression: unknown;
  lookAt: unknown;
};
