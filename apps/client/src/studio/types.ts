export type Tool = "translate" | "rotate" | "scale";

export enum DND_TYPES {
  TreeNode = "TreeNode",
}

export interface UserData {
  treeNode?: boolean;
  siblingIndex?: number;
}
