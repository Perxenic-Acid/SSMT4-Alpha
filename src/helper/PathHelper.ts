import { ElMessage } from 'element-plus';
import { join } from '@tauri-apps/api/path';
import { AppStateManager } from '../store/AppStateManager';
import { GlobalConfig } from '../store/GlobalConfig';
import { ResourceManager } from '../store/ResourceManager';
import { mkdir } from '@tauri-apps/plugin-fs';
import { i18n } from '../i18n';

const t = i18n.global.t;

export class PathHelper {
	static async GetGame3DmigotoFolderPath(gameName: string): Promise<string | undefined> {
		const normalizedGameName = gameName.trim();
		if (!normalizedGameName || normalizedGameName === 'Default') {
			ElMessage.info(t('pathHelper.messages.selectGameConfigFirst'));
			return undefined;
		}

		const config = await ResourceManager.loadGameConfig(normalizedGameName);
		const installDir = (config?.installDir || '').trim();
		if (installDir) {
			return installDir;
		}

		const cacheRoot = await GlobalConfig.SSMT4CustomCacheFolder();
		return await join(cacheRoot, '3Dmigoto', normalizedGameName);
	}

	static async GetCurrentGame3DmigotoFolderPath(): Promise<string | undefined> {
		const gameName = AppStateManager.appSettings.CurrentGameName;
		if (!gameName) {
			ElMessage.info(t('pathHelper.messages.selectGameConfigFirst'));
			return undefined;
		}

		return await this.GetGame3DmigotoFolderPath(gameName);
	}

    static async GetWorkspaceGeneratedModFolderPath(workspaceName: string): Promise<string | undefined> {
        const current3dmigotoFolderPath = await this.GetCurrentGame3DmigotoFolderPath();
        if (!current3dmigotoFolderPath) {
			ElMessage.info(t('pathHelper.messages.selectGameConfigAndEnsureMigotoPath'));
            return undefined;
        }
        
        const workspaceGeneratedModFolderPath = await join(current3dmigotoFolderPath, 'Mods', 'SSMTGeneratedMod', workspaceName.trim());
        await mkdir(workspaceGeneratedModFolderPath, { recursive: true });
        return workspaceGeneratedModFolderPath;
    }
}
