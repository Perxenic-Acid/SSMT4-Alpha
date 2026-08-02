import type { ModInfo } from './ModsManagement.types';
import { normalizeManualOrderId } from './ModsManagement.paths';

export const hashString = (input: string) => {
    let hash = 0;
    for (let i = 0; i < input.length; i += 1) {
        hash = ((hash << 5) - hash) + input.charCodeAt(i);
        hash |= 0;
    }
    return Math.abs(hash);
};

export const getModDynamicStyle = (mod: ModInfo) => {
    const seed = hashString(normalizeManualOrderId(mod.id || mod.relativePath) || mod.name || 'mod');
    const phaseA = -((seed % 4200) / 1000);
    const phaseB = -(((seed >> 3) % 5800) / 1000);
    const breath = 3.8 + ((seed % 9) * 0.17);
    const sheen = 4.6 + (((seed >> 2) % 8) * 0.22);
    const rotate = 7.4 + (((seed >> 5) % 9) * 0.26);

    return {
        '--phase-a': `${phaseA.toFixed(2)}s`,
        '--phase-b': `${phaseB.toFixed(2)}s`,
        '--breath-duration': `${breath.toFixed(2)}s`,
        '--sheen-duration': `${sheen.toFixed(2)}s`,
        '--rotate-duration': `${rotate.toFixed(2)}s`,
    } as Record<string, string>;
};

export const getClipboardImageExtension = (mimeType: string) => {
    switch (mimeType.toLowerCase()) {
        case 'image/jpeg':
            return 'jpg';
        case 'image/png':
            return 'png';
        case 'image/webp':
            return 'webp';
        case 'image/gif':
            return 'gif';
        case 'image/bmp':
            return 'bmp';
        default:
            return 'png';
    }
};

export const getPreviewFileNameLower = (path: string) => (path.split(/[\\/]/).pop() || '').toLowerCase();
