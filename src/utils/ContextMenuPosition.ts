export type ContextMenuPositionInput = {
  clientX: number;
  clientY: number;
  menuWidth: number;
  menuHeight: number;
  edgeGap?: number;
};

export type ContextMenuPositionResult = {
  x: number;
  y: number;
};

export const calculateContextMenuPosition = (
  input: ContextMenuPositionInput
): ContextMenuPositionResult => {
  const edgeGap = input.edgeGap ?? 8;
  const yOffset = 30;

  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  const desiredX = input.clientX;
  const desiredY = input.clientY;

  const maxX = Math.max(edgeGap, viewportWidth - input.menuWidth - edgeGap);
  const maxY = Math.max(edgeGap, viewportHeight - input.menuHeight - edgeGap);

  const clampedX = Math.max(edgeGap, Math.min(desiredX, maxX));
  const clampedY = Math.max(edgeGap, Math.min(desiredY, maxY));
  const shiftedY = clampedY - yOffset;

  return {
    x: clampedX,
    y: Math.max(edgeGap, Math.min(shiftedY, maxY)),
  };
};
