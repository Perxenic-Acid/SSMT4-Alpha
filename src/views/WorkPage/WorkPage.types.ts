export type ModelRow = {
  drawIB: string;
  aliasName: string;
};

export type SkipRow = {
  skipIB: string;
  aliasName: string;
  indexCount: string;
  firstIndex: string;
};

export type VSCheckRow = {
  enabled: boolean;
  hash: string;
};

export type DrawerCollapsedState = {
  workspace: boolean;
  workspaceSelector: boolean;
  commonFolders: boolean;
  textureExtract: boolean;
  otherFunctions: boolean;
};
