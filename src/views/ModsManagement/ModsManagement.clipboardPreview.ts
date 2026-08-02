import type { ImageSize } from '@tauri-apps/api/image';

type ClipboardImageLike = {
    size(): Promise<ImageSize>;
    rgba(): Promise<Uint8Array>;
};

/**
 * Convert a Tauri clipboard Image into PNG bytes for saving as a preview file.
 *
 * The clipboard plugin returns a Tauri Image (a resource handle), not raw
 * bytes.  We must go through RGBA → canvas → PNG because the image resource
 * only exposes RGBA pixel data, not the original encoded format.
 */
export const clipboardImageToPngBytes = async (image: ClipboardImageLike): Promise<Uint8Array> => {
    // Serialize IPC calls — reading size and RGBA concurrently on the
    // same resource handle can race on some Tauri backends.
    const size = await image.size();
    const rgba = await image.rgba();

    const width = size.width;
    const height = size.height;

    if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
        throw new Error('Clipboard image has invalid dimensions');
    }

    const expectedLen = width * height * 4;
    if (rgba.byteLength < expectedLen) {
        throw new Error(
            `Clipboard image RGBA data too small: got ${rgba.byteLength} bytes, expected ${expectedLen} for ${width}×${height}`
        );
    }

    // Use a canvas to re-encode as PNG.
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Failed to create canvas 2d context');

    // ImageData requires exactly width*height*4 bytes — trim if there is trailing padding.
    const imageDataArray = rgba.byteLength === expectedLen
        ? new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, expectedLen)
        : new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, expectedLen);
    const imageData = new ImageData(imageDataArray, width, height);
    ctx.putImageData(imageData, 0, 0);

    const blob = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob((result) => {
            if (result) resolve(result);
            else reject(new Error('Failed to encode clipboard image as PNG'));
        }, 'image/png');
    });

    return new Uint8Array(await blob.arrayBuffer());
};
