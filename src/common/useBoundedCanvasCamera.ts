import { computed, nextTick, ref } from 'vue';
import type { Ref } from 'vue';

export interface CanvasView {
  x: number;
  y: number;
  scale: number;
}

export interface CanvasBounds {
  width: number;
  height: number;
}

interface UseBoundedCanvasCameraOptions {
  viewportRef: Ref<HTMLElement | null>;
  storageKey: string;
  worldWidth: number;
  worldHeight: number;
  minScale: number;
  maxScale: number;
  scaleStep: number;
  edgeReveal?: number;
  resetPadding?: number;
  warningLabel?: string;
  getResetBounds?: () => CanvasBounds;
  shouldPanFromTarget?: (target: EventTarget | null) => boolean;
  shouldHandleWheel?: (event: WheelEvent) => boolean;
  onViewChange?: () => void;
}

const DEFAULT_EDGE_REVEAL = 32;
const DEFAULT_RESET_PADDING = 28;

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const finiteOr = (value: unknown, fallback: number) => (
  typeof value === 'number' && Number.isFinite(value) ? value : fallback
);

export const useBoundedCanvasCamera = (options: UseBoundedCanvasCameraOptions) => {
  const worldWidth = Math.max(1, options.worldWidth);
  const worldHeight = Math.max(1, options.worldHeight);
  const edgeReveal = Math.max(0, options.edgeReveal ?? DEFAULT_EDGE_REVEAL);
  const resetPadding = Math.max(0, options.resetPadding ?? DEFAULT_RESET_PADDING);

  const clampScale = (scale: unknown) => clamp(
    finiteOr(typeof scale === 'number' ? scale : Number(scale), 1),
    options.minScale,
    options.maxScale,
  );

  const normalizeView = (view: CanvasView): CanvasView => ({
    x: Math.round(finiteOr(view.x, 0)),
    y: Math.round(finiteOr(view.y, 0)),
    scale: Number(clampScale(view.scale).toFixed(3)),
  });

  const getViewportBounds = (): CanvasBounds | null => {
    const viewport = options.viewportRef.value;
    if (!viewport) return null;
    const rect = viewport.getBoundingClientRect();
    const width = rect.width || viewport.clientWidth;
    const height = rect.height || viewport.clientHeight;
    if (width <= 0 || height <= 0) return null;
    return { width, height };
  };

  const constrainAxis = (offset: number, scaledWorldSize: number, viewportSize: number) => {
    const minOffset = viewportSize - edgeReveal - scaledWorldSize;
    const maxOffset = edgeReveal;
    if (minOffset > maxOffset) {
      return (viewportSize - scaledWorldSize) / 2;
    }
    return clamp(offset, minOffset, maxOffset);
  };

  const constrainView = (view: CanvasView): CanvasView => {
    const normalized = normalizeView(view);
    const viewport = getViewportBounds();
    if (!viewport) return normalized;

    return normalizeView({
      x: constrainAxis(normalized.x, worldWidth * normalized.scale, viewport.width),
      y: constrainAxis(normalized.y, worldHeight * normalized.scale, viewport.height),
      scale: normalized.scale,
    });
  };

  const loadCanvasView = (): CanvasView => {
    try {
      const raw = localStorage.getItem(options.storageKey);
      if (!raw) return { x: 0, y: 0, scale: 1 };
      const saved = JSON.parse(raw) as Partial<CanvasView>;
      return normalizeView({
        x: finiteOr(saved.x, 0),
        y: finiteOr(saved.y, 0),
        scale: clampScale(saved.scale),
      });
    } catch (error) {
      console.warn(`Failed to load ${options.warningLabel || 'canvas'} view`, error);
      return { x: 0, y: 0, scale: 1 };
    }
  };

  const canvasView = ref<CanvasView>(loadCanvasView());
  const isCanvasPanning = ref(false);
  let canvasPanStart: { pointerId: number; clientX: number; clientY: number; x: number; y: number } | null = null;

  const saveCanvasView = (view: CanvasView) => {
    localStorage.setItem(options.storageKey, JSON.stringify(view));
  };

  const setCanvasView = (view: CanvasView) => {
    const nextView = constrainView(view);
    canvasView.value = nextView;
    saveCanvasView(nextView);
    options.onViewChange?.();
  };

  const syncCanvasViewBounds = () => {
    setCanvasView(canvasView.value);
  };

  const canvasTransformStyle = computed(() => ({
    transform: `translate3d(${canvasView.value.x}px, ${canvasView.value.y}px, 0) scale(${canvasView.value.scale})`,
    width: `${worldWidth}px`,
    height: `${worldHeight}px`,
  }));

  const canvasScalePercent = computed(() => `${Math.round(canvasView.value.scale * 100)}%`);

  const zoomCanvasAt = (clientX: number, clientY: number, nextScale: number) => {
    const viewportElement = options.viewportRef.value;
    if (!viewportElement) {
      setCanvasView({ ...canvasView.value, scale: nextScale });
      return;
    }

    const rect = viewportElement.getBoundingClientRect();
    const localX = clientX - rect.left;
    const localY = clientY - rect.top;
    const current = canvasView.value;
    const worldX = (localX - current.x) / current.scale;
    const worldY = (localY - current.y) / current.scale;
    const clampedScale = clampScale(nextScale);

    setCanvasView({
      x: localX - worldX * clampedScale,
      y: localY - worldY * clampedScale,
      scale: clampedScale,
    });
  };

  const zoomCanvasBy = (delta: number) => {
    const viewport = options.viewportRef.value;
    const rect = viewport?.getBoundingClientRect();
    zoomCanvasAt(
      rect ? rect.left + rect.width / 2 : window.innerWidth / 2,
      rect ? rect.top + rect.height / 2 : window.innerHeight / 2,
      canvasView.value.scale + delta,
    );
  };

  const resetCanvasView = () => {
    const viewport = getViewportBounds();
    if (!viewport) {
      setCanvasView({ x: 0, y: 0, scale: 1 });
      return;
    }

    const bounds = options.getResetBounds?.() ?? { width: worldWidth, height: worldHeight };
    const scale = Math.min(
      1,
      (viewport.width - resetPadding * 2) / Math.max(bounds.width, 1),
      (viewport.height - resetPadding * 2) / Math.max(bounds.height, 1),
    );
    const nextScale = clampScale(scale);

    setCanvasView({
      x: Math.max(resetPadding, (viewport.width - bounds.width * nextScale) / 2),
      y: Math.max(resetPadding, (viewport.height - bounds.height * nextScale) / 2),
      scale: nextScale,
    });
  };

  const centerCanvasView = () => {
    const viewport = getViewportBounds();
    if (!viewport) {
      setCanvasView({ ...canvasView.value, x: 0, y: 0 });
      return;
    }

    const bounds = options.getResetBounds?.() ?? { width: worldWidth, height: worldHeight };
    const scale = canvasView.value.scale;

    setCanvasView({
      x: (viewport.width - bounds.width * scale) / 2,
      y: (viewport.height - bounds.height * scale) / 2,
      scale,
    });
  };

  const onCanvasPointerDown = (event: PointerEvent) => {
    const canPanFromTarget = options.shouldPanFromTarget?.(event.target) ?? true;
    if (event.button !== 1 && (event.button !== 0 || !canPanFromTarget)) return;
    event.preventDefault();
    isCanvasPanning.value = true;
    canvasPanStart = {
      pointerId: event.pointerId,
      clientX: event.clientX,
      clientY: event.clientY,
      x: canvasView.value.x,
      y: canvasView.value.y,
    };
    options.viewportRef.value?.setPointerCapture(event.pointerId);
  };

  const onCanvasPointerMove = (event: PointerEvent) => {
    if (!canvasPanStart || canvasPanStart.pointerId !== event.pointerId) return;
    event.preventDefault();
    setCanvasView({
      x: canvasPanStart.x + event.clientX - canvasPanStart.clientX,
      y: canvasPanStart.y + event.clientY - canvasPanStart.clientY,
      scale: canvasView.value.scale,
    });
  };

  const stopCanvasPanning = (event?: PointerEvent) => {
    if (event && canvasPanStart?.pointerId === event.pointerId) {
      options.viewportRef.value?.releasePointerCapture(event.pointerId);
    }
    isCanvasPanning.value = false;
    canvasPanStart = null;
  };

  const onCanvasWheel = (event: WheelEvent) => {
    if (options.shouldHandleWheel && !options.shouldHandleWheel(event)) return;
    event.preventDefault();
    const zoomDelta = event.deltaY < 0 ? options.scaleStep : -options.scaleStep;
    zoomCanvasAt(event.clientX, event.clientY, canvasView.value.scale + zoomDelta);
  };

  void nextTick(syncCanvasViewBounds);

  return {
    canvasView,
    isCanvasPanning,
    canvasTransformStyle,
    canvasScalePercent,
    setCanvasView,
    syncCanvasViewBounds,
    zoomCanvasAt,
    zoomCanvasBy,
    resetCanvasView,
    centerCanvasView,
    onCanvasPointerDown,
    onCanvasPointerMove,
    stopCanvasPanning,
    onCanvasWheel,
  };
};
