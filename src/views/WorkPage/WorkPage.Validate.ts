import { i18n } from '../../i18n';

const t = i18n.global.t;

export type ExtractModelInputRow = {
  drawIB: string;
  aliasName: string;
};

export type ExtractModelPayloadRow = {
  drawIB: string;
  aliasName: string;
};

export const buildExtractModelPayload = (
  rows: ExtractModelInputRow[]
): ExtractModelPayloadRow[] => {
  return rows
    .map((row) => ({
      drawIB: row.drawIB.trim(),
      aliasName: row.aliasName.trim(),
    }))
    .filter((row) => row.drawIB !== '' || row.aliasName !== '');
};

export const validateExtractModelPayloadNotEmpty = (
  payload: ExtractModelPayloadRow[]
): string | undefined => {
  if (payload.length === 0) {
    return t('workPage.messages.fillOneDrawIBOrAliasName');
  }
  return undefined;
};

export const validateFrameAnalysisFolderPath = (
  frameAnalysisFolderPath: string
): string | undefined => {
  if (!frameAnalysisFolderPath || frameAnalysisFolderPath.trim() === '') {
    return t('workPage.messages.frameAnalysisFolderNotFound');
  }
  return undefined;
};

export const resolveLodNameFromFrameAnalysisPath = (
  frameAnalysisFolderPath: string
): string | undefined => {
  const trimmedPath = frameAnalysisFolderPath.trim();
  if (!trimmedPath) {
    return undefined;
  }

  const normalizedPath = trimmedPath.replace(/\\+/g, '/').replace(/\/+$/g, '');
  const pathSegments = normalizedPath.split('/').filter(Boolean);

  for (let index = pathSegments.length - 1; index >= 0; index -= 1) {
    const segment = pathSegments[index]?.trim();
    if (segment && /^lod\d+$/i.test(segment)) {
      return segment;
    }
  }

  const lodName = pathSegments[pathSegments.length - 1]?.trim();

  return lodName || undefined;
};

export const resolveLodName = (
  selectedFrameAnalysis: string,
  frameAnalysisFolderPath: string
): string | undefined => {
  const trimmedSelectedFrameAnalysis = selectedFrameAnalysis.trim();
  if (/^lod\d+$/i.test(trimmedSelectedFrameAnalysis)) {
    return trimmedSelectedFrameAnalysis;
  }

  return resolveLodNameFromFrameAnalysisPath(frameAnalysisFolderPath);
};
