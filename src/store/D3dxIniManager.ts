import { exists, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { i18n } from '../i18n'

const t = i18n.global.t

export type IniLines = string[]

export interface IniSectionEntry {
  key: string
  value: string
  lineIndex: number
}

export class D3dxIniManager {
  static async loadIni(path: string): Promise<IniLines> {
    if (!(await exists(path))) {
      throw new Error(t('d3dxIniManager.messages.iniNotFoundAt', { path }))
    }
    const content = await readTextFile(path)
    return content.split(/\r?\n/)
  }

  static async saveIni(path: string, lines: IniLines): Promise<void> {
    const content = lines.join('\n')
    await writeTextFile(path, content)
  }

  static setIniValue(lines: IniLines, section: string, key: string, value: string): IniLines {
    const sectionLower = section.toLowerCase()
    const keyLower = key.toLowerCase()
    let inSection = false
    let keyFound = false
    let sectionStart = -1
    let sectionEnd = -1

    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i]
      const trimmed = raw.trim()
      const clean = trimmed.split(';')[0].trim()
      if (clean.startsWith('[') && clean.endsWith(']')) {
        const name = clean.slice(1, -1).trim().toLowerCase()
        if (inSection) {
          sectionEnd = i - 1
          inSection = false
        }
        if (name === sectionLower) {
          inSection = true
          sectionStart = i
        }
        continue
      }

      if (inSection) {
        if (trimmed.startsWith(';') || trimmed.startsWith('#') || trimmed === '') continue
        const eq = raw.indexOf('=')
        if (eq !== -1) {
          const currentKey = raw.slice(0, eq).trim().toLowerCase()
          if (currentKey === keyLower) {
            const prefix = raw.slice(0, eq)
            lines[i] = `${prefix.trimEnd()} = ${value}`
            keyFound = true
            break
          }
        }
      }
    }

    if (!keyFound) {
      let insertPos = lines.length
      if (sectionStart !== -1) {
        if (sectionEnd === -1 && inSection) {
          insertPos = lines.length
        } else if (sectionEnd !== -1) {
          insertPos = sectionEnd + 1
        } else {
          insertPos = lines.length
        }
        lines.splice(insertPos, 0, `${key} = ${value}`)
      } else {
        lines.push('', `[${section}]`, `${key} = ${value}`)
      }
    }

    return lines
  }

  static removeIniKey(lines: IniLines, section: string, key: string): IniLines {
    const sectionLower = section.toLowerCase()
    const keyLower = key.toLowerCase()
    let inSection = false
    const toRemove: number[] = []

    for (let i = 0; i < lines.length; i++) {
      const raw = lines[i]
      const trimmed = raw.trim()
      const clean = trimmed.split(';')[0].trim()

      if (clean.startsWith('[') && clean.endsWith(']')) {
        const name = clean.slice(1, -1).trim().toLowerCase()
        inSection = name === sectionLower
        continue
      }

      if (!inSection) continue
      if (trimmed.startsWith(';') || trimmed.startsWith('#') || trimmed === '') continue
      const eq = raw.indexOf('=')
      if (eq !== -1) {
        const currentKey = raw.slice(0, eq).trim().toLowerCase()
        if (currentKey === keyLower) {
          toRemove.push(i)
        }
      }
    }

    for (let i = toRemove.length - 1; i >= 0; i--) {
      lines.splice(toRemove[i], 1)
    }

    return lines
  }

  static getSectionEntries(lines: IniLines, section: string): IniSectionEntry[] {
    const sectionLower = section.toLowerCase()
    let inSection = false
    const entries: IniSectionEntry[] = []

    for (let i = 0; i < lines.length; i += 1) {
      const raw = lines[i]
      const trimmed = raw.trim()
      const clean = trimmed.split(';')[0].trim()

      if (clean.startsWith('[') && clean.endsWith(']')) {
        const name = clean.slice(1, -1).trim().toLowerCase()
        inSection = name === sectionLower
        continue
      }

      if (!inSection) continue
      if (!trimmed || trimmed.startsWith(';') || trimmed.startsWith('#')) continue

      const eq = raw.indexOf('=')
      if (eq === -1) continue

      entries.push({
        key: raw.slice(0, eq).trim(),
        value: raw.slice(eq + 1).trim(),
        lineIndex: i,
      })
    }

    return entries
  }

  static getIniValue(lines: IniLines, section: string, key: string): string | undefined {
    const keyLower = key.toLowerCase()
    const entry = this.getSectionEntries(lines, section).find(item => item.key.toLowerCase() === keyLower)
    return entry?.value
  }

  static setSectionLineEnabled(lines: IniLines, section: string, targetLine: string, enabled: boolean): IniLines {
    const sectionLower = section.toLowerCase()
    const normalizedTargetLine = targetLine.trim()
    let sectionStart = -1
    let sectionEnd = lines.length - 1

    for (let index = 0; index < lines.length; index += 1) {
      const raw = lines[index]
      const trimmed = raw.trim()
      const clean = trimmed.split(';')[0].trim()

      if (!clean.startsWith('[') || !clean.endsWith(']')) {
        continue
      }

      const name = clean.slice(1, -1).trim().toLowerCase()
      if (name === sectionLower) {
        sectionStart = index
        for (let nextIndex = index + 1; nextIndex < lines.length; nextIndex += 1) {
          const nextTrimmed = lines[nextIndex].trim()
          const nextClean = nextTrimmed.split(';')[0].trim()
          if (nextClean.startsWith('[') && nextClean.endsWith(']')) {
            sectionEnd = nextIndex - 1
            break
          }
        }
        break
      }
    }

    const matchingLineIndexes: number[] = []

    if (sectionStart !== -1) {
      for (let index = sectionStart + 1; index <= sectionEnd; index += 1) {
        const trimmed = lines[index].trim()
        if (!trimmed) {
          continue
        }

        const uncommented = trimmed.replace(/^[;#]+\s*/, '').trim()
        if (uncommented === normalizedTargetLine) {
          matchingLineIndexes.push(index)
        }
      }
    }

    if (matchingLineIndexes.length > 0) {
      matchingLineIndexes.forEach(index => {
        lines[index] = enabled ? normalizedTargetLine : `;${normalizedTargetLine}`
      })
      return lines
    }

    if (!enabled) {
      return lines
    }

    if (sectionStart === -1) {
      lines.push('', `[${section}]`, normalizedTargetLine)
      return lines
    }

    lines.splice(sectionEnd + 1, 0, normalizedTargetLine)
    return lines
  }

  static setIniValues(lines: IniLines, section: string, values: Record<string, string>): IniLines {
    Object.entries(values).forEach(([key, value]) => {
      lines = D3dxIniManager.setIniValue(lines, section, key, value)
    })
    return lines
  }
}
