import { exists, mkdir } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path';
import { readDir } from '@tauri-apps/plugin-fs';
import { readFile,writeFile } from '@tauri-apps/plugin-fs';

export class SSMTFileUtils {

    public static async CreateFolderIfNotExists(path: string) {
        if (!(await exists(path))) {
            await mkdir(path, { recursive: true })
        }
    }


    public static async CopyFileIfMissing(src: string, dst: string) {
        if (await exists(dst)) return
        const data = await readFile(src)
        await writeFile(dst, data)
    }


    public static async CopyDirRecursive(src: string, dst: string): Promise<void> {
        await SSMTFileUtils.CreateFolderIfNotExists(dst)
        const entries = await readDir(src)
        for (const entry of entries) {
            const srcPath = await join(src, entry.name)
            const dstPath = await join(dst, entry.name)
            if (entry.isDirectory) {
                await SSMTFileUtils.CopyDirRecursive(srcPath, dstPath)
            } else {
            await SSMTFileUtils.CopyFileIfMissing(srcPath, dstPath)
            }
        }
    }

    // Lightweight path join that keeps forward slashes and trims duplicates.
    public static async JoinPath(...parts: string[]): Promise<string> {
        return parts.map((p, i) => {
                const normalized = p.replace(/\\/g, '/');
                if (i === 0) return normalized.replace(/\/+$|^\/+/, '');
                return normalized.replace(/^\/+|\/+$/g, '');
            })
            .filter(Boolean)
            .join('/');
    }

}