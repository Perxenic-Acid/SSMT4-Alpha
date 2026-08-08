<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ChatDotRound,
  CopyDocument,
  Delete,
  Link as LinkIcon,
  MagicStick,
  Promotion,
  Setting,
  VideoPause,
} from '@element-plus/icons-vue'
import { fetch } from '@tauri-apps/plugin-http'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { openUrl } from '@tauri-apps/plugin-opener'
import { getVersion } from '@tauri-apps/api/app'
import { AppStateManager } from '../../store/AppStateManager'
import { mcpTools, validateToolArgs, MCP_CATEGORY_LABELS } from '../../store/XianZunMcp'
import type { McpTool, RiskLevel } from '../../store/XianZunMcp'
import { buildCapabilityTools } from '../../store/XianZunCapabilities'
import type { CapabilityTool } from '../../store/XianZunCapabilities'
import { useResourceManagerStore } from '../../store/ResourceManager'
import { useModManagerStore } from '../../store/ModManager'
import { useModTagStore } from '../../store/ModTagStore'
import { useModPresetStore } from '../../store/ModPresetStore'
import { useModStateStore } from '../../store/ModStateStore'
import { useGameConfigStore } from '../../store/GameConfig'

/* ═══════════════════════════════════════════════
   Types
   ═══════════════════════════════════════════════ */

interface ChatMessage {
  id: string
  role: 'user' | 'assistant' | 'error'
  content: string
  streaming?: boolean
  createdAt: number
  toolEvents?: ToolEvent[]
}

interface ToolEvent {
  command: string
  arguments: Record<string, unknown>
  result: string
  ok: boolean
}

interface XianZunCommand {
  name: string
  description: string
  inputSchema: {
    type: 'object'
    properties: Record<string, { type: string; description: string; enum?: string[] }>
    required: string[]
  }
  risk?: RiskLevel
  execute: (args: Record<string, unknown>) => string | Promise<string>
}

interface ApiMessage {
  role: 'system' | 'user' | 'assistant' | 'tool'
  content: string
}

/* ═══════════════════════════════════════════════
   App wiring
   ═══════════════════════════════════════════════ */

const appSettings = AppStateManager.appSettings
const router = useRouter()
const { t } = useI18n()

const STORAGE_KEY = 'xianzun.messages.v1'
const MAX_TOOL_ROUNDS = 10
const STREAM_TEMPERATURE = 0.8

/* ═══════════════════════════════════════════════
   Command registry — every app capability is
   registered here as an MCP-style tool schema so
   the agent can discover and call it. UI commands
   live below; all 39 Tauri commands come from
   XianZunMcp.ts (invoke-backed MCP tools).
   ═══════════════════════════════════════════════ */

const PAGE_MAP: Record<string, string> = {
  home: '/',
  games: '/games',
  mods: '/mods',
  gamebanana: '/gamebanana',
  nexusmods: '/nexusmods',
  work: '/work',
  'mark-texture-full': '/mark-texture-full',
  settings: '/settings',
  xianzun: '/xianzun',
}

const stringProp = (description: string, enumValues?: string[]) => ({
  type: 'string',
  description,
  ...(enumValues ? { enum: enumValues } : {}),
})

const uiCommands: XianZunCommand[] = [
  {
    name: 'navigate_to_page',
    description: '跳转到 SSMT4 的指定页面:主页、游戏库、模组管理、GameBanana、NexusMods、工作台、提取后处理、设置、小尊小尊。',
    inputSchema: {
      type: 'object',
      properties: {
        page: stringProp('目标页面 id', Object.keys(PAGE_MAP)),
      },
      required: ['page'],
    },
    execute: (args) => {
      const page = String(args.page ?? '').trim().toLowerCase()
      const path = PAGE_MAP[page]
      if (!path) {
        return `未知页面 "${page}"。可用页面:${Object.keys(PAGE_MAP).join(', ')}`
      }
      void router.push(path)
      return `已跳转到页面 "${page}"(${path})`
    },
  },
  {
    name: 'get_app_state',
    description: '获取当前应用状态:当前选择的游戏、应用版本、可用页面列表。',
    inputSchema: { type: 'object', properties: {}, required: [] },
    execute: async () => {
      let version = 'unknown'
      try {
        version = await getVersion()
      } catch {
        // ignore — version is best-effort
      }
      return JSON.stringify(
        {
          currentGame: appSettings.CurrentGameName || 'Default',
          appVersion: version,
          pages: Object.keys(PAGE_MAP),
        },
        null,
        2,
      )
    },
  },
  {
    name: 'list_capabilities',
    description: '列出小尊小尊当前可以调用的全部指令(名称、参数、风险级别)。自动注册的模块函数名称格式为 模块.函数,调用前可先用本指令查询。',
    inputSchema: { type: 'object', properties: {}, required: [] },
    execute: () => {
      return commands
        .map(
          (c) =>
            `${c.name}(${c.inputSchema.required.join(', ')})${c.risk && c.risk !== 'read' ? ` [${c.risk}]` : ''}: ${c.description}`,
        )
        .join('\n')
    },
  },
  {
    name: 'get_tool_schema',
    description: '查询某个指令的完整参数 schema(必填/可选参数、参数说明、风险级别)。',
    inputSchema: {
      type: 'object',
      properties: {
        toolName: stringProp('要查询的指令名称'),
      },
      required: ['toolName'],
    },
    execute: (args) => {
      const toolName = String(args.toolName ?? '').trim()
      const target = commands.find((c) => c.name === toolName)
      if (!target) {
        return `未知指令: ${toolName}。可用:list_capabilities 查询全部。`
      }
      return JSON.stringify(
        {
          name: target.name,
          description: target.description,
          risk: target.risk ?? 'read',
          inputSchema: target.inputSchema,
        },
        null,
        2,
      )
    },
  },
  {
    name: 'clear_conversation',
    description: '清空当前对话历史。',
    inputSchema: { type: 'object', properties: {}, required: [] },
    execute: () => {
      messages.value = []
      persist()
      return '对话已清空'
    },
  },
]

// UI commands + all Tauri commands (MCP tools) + every frontend
// module function (auto-registered capabilities).
const capabilityTools = buildCapabilityTools({
  resourceManager: useResourceManagerStore() as unknown as Record<string, unknown>,
  modManager: useModManagerStore() as unknown as Record<string, unknown>,
  modTagStore: useModTagStore() as unknown as Record<string, unknown>,
  modPresetStore: useModPresetStore() as unknown as Record<string, unknown>,
  modStateStore: useModStateStore() as unknown as Record<string, unknown>,
  gameConfig: useGameConfigStore() as unknown as Record<string, unknown>,
})
const commands: XianZunCommand[] = [...uiCommands, ...mcpTools, ...capabilityTools]

/* ═══════════════════════════════════════════════
   Chat state
   ═══════════════════════════════════════════════ */

const messages = ref<ChatMessage[]>([])
const draft = ref('')
const isStreaming = ref(false)
const settingsOpen = ref(false)
const testing = ref(false)
const expandedTools = ref<string[]>([])
const inputRef = ref<HTMLTextAreaElement | null>(null)
const chatListRef = ref<HTMLElement | null>(null)
let abortController: AbortController | null = null
let idCounter = 0

const nextId = () => `xz-${Date.now()}-${idCounter++}`

const lastAssistant = computed(() => {
  for (let i = messages.value.length - 1; i >= 0; i--) {
    if (messages.value[i].role === 'assistant') return messages.value[i]
  }
  return null
})

const waitingFirstToken = computed(() => isStreaming.value && !lastAssistant.value?.content)

const statusText = computed(() => {
  if (isStreaming.value) return t('xianzun.streaming')
  if (!appSettings.xianzunApiKey.trim()) return t('xianzun.offline')
  return t('xianzun.online')
})

const statusClass = computed(() => {
  if (isStreaming.value) return 'streaming'
  if (!appSettings.xianzunApiKey.trim()) return 'offline'
  return 'online'
})

const suggestionList = computed(() => {
  const raw = t('xianzun.suggestions') as unknown
  return Array.isArray(raw) ? (raw as string[]) : []
})

const capabilityGroups = computed(() => {
  const groups = new Map<string, XianZunCommand[]>()
  for (const cmd of commands) {
    const key = (cmd as McpTool | CapabilityTool).category ?? 'other'
    const bucket = groups.get(key)
    if (bucket) bucket.push(cmd)
    else groups.set(key, [cmd])
  }
  return Array.from(groups.entries()).map(([key, tools]) => ({
    key,
    label: MCP_CATEGORY_LABELS[key] ?? key,
    tools,
  }))
})

/* ═══════════════════════════════════════════════
   Persistence
   ═══════════════════════════════════════════════ */

const persist = () => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(messages.value))
  } catch {
    // storage may be unavailable — chat still works in memory
  }
}

const loadMessages = () => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) {
      messages.value = parsed.filter(
        (m: unknown): m is ChatMessage =>
          !!m &&
          typeof m === 'object' &&
          ((m as ChatMessage).role === 'user' ||
            (m as ChatMessage).role === 'assistant' ||
            (m as ChatMessage).role === 'error') &&
          typeof (m as ChatMessage).content === 'string',
      )
    }
  } catch {
    // corrupt storage — start fresh
  }
}

/* ═══════════════════════════════════════════════
   System prompt (persona + command registry)
   ═══════════════════════════════════════════════ */

const buildSystemPrompt = (): string => {
  // Precise tools (UI + Tauri commands) are listed inline; auto-registered
  // module functions are discovered on demand via list_capabilities to keep
  // the system prompt compact.
  const commandList = [...uiCommands, ...mcpTools]
    .map((c) => {
      const requiredParams = c.inputSchema.required.join(', ')
      const optionalParams = Object.keys(c.inputSchema.properties).filter(
        (key) => !c.inputSchema.required.includes(key),
      )
      const optionalText = optionalParams.length > 0 ? `, 可选:${optionalParams.join(', ')}` : ''
      const riskText = c.risk && c.risk !== 'read' ? ` [${c.risk === 'danger' ? '危险' : '写'}]` : ''
      return `- ${c.name}(${requiredParams}${optionalText})${riskText}: ${c.description}`
    })
    .join('\n')

  const base = [
    '你是「小尊小尊」(XianZun),SSMT4 模型工具内置的 AI 智能体。你亲切、专业、表达简洁,始终使用用户提问所用的语言回复。',
    '',
    '你拥有操控整个应用的能力(如同自己的手臂):不仅能调用下方精确注册的指令,还能调用前端全部模块函数(自动注册,名称格式为 模块.函数,例如 ResourceManager.loadGameConfig、ModManager.toggleMod、MigotoManager.switchD3d11Mode、PathHelper.GetCurrentGame3DmigotoFolderPath)。',
    '',
    '精确注册的指令(参数键名必须与指令参数名一致):',
    commandList,
    '',
    '调用规则:',
    '- 当你需要操作应用时,在回复中输出一个语言标记为 tool_call 的 fenced code block,内容为 JSON: {"command":"指令名","arguments":{...}}。',
    '- 对自动注册的模块函数,先用 list_capabilities 查看函数名与参数,再用 get_tool_schema 查看详细参数说明,然后调用。',
    '- 你可以自由组合多个指令完成复杂任务(例如:扫描 Mod 库 → 从 GameBanana 下载指定类型 Mod → 安装 → 打标签 → 启动游戏),每一步的执行结果都会回传给你,根据结果决定下一步。',
    '- 缺少必需参数(如 installDir、frameAnalysisFolder、drawIb hash、downloadUrl 等用户才知道的信息)时,不要猜测或编造,先向用户提问,补齐后再调用。',
    '- 标记 [写] 或 [危险] 的指令会弹出确认框征求用户同意;若用户拒绝(返回"用户拒绝"),不要硬重试,改为向用户说明或换一种方案。',
    '- 调用可能耗时较长的命令(下载、全量提取、扫描)前,先告诉用户你正在做什么。',
    '- 严禁编造指令执行结果;只有收到工具返回后才可以引用其结果。',
  ].join('\n')

  const custom = appSettings.xianzunSystemPrompt?.trim()
  return custom ? `${custom}\n\n${base}` : base
}

/* ═══════════════════════════════════════════════
   DeepSeek streaming (OpenAI-compatible SSE)
   ═══════════════════════════════════════════════ */

const isAbortError = (err: unknown): boolean => {
  if (err instanceof Error) {
    return (
      err.name === 'AbortError' ||
      err.name === 'Canceled' ||
      /cancelled|canceled/i.test(err.message)
    )
  }
  return false
}

const errorText = (err: unknown): string => {
  if (err instanceof Error) return err.message
  return String(err)
}

const streamChatCompletion = async (opts: {
  apiUrl: string
  apiKey: string
  model: string
  messages: ApiMessage[]
  signal: AbortSignal
  onDelta: (delta: string) => void
}): Promise<string> => {
  const base = opts.apiUrl.trim().replace(/\/+$/, '')
  const url = `${base}/chat/completions`

  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${opts.apiKey}`,
    },
    body: JSON.stringify({
      model: opts.model,
      messages: opts.messages,
      stream: true,
      temperature: STREAM_TEMPERATURE,
    }),
    signal: opts.signal,
  })

  if (!res.ok) {
    let detail = ''
    try {
      const raw = await res.text()
      try {
        const parsed = JSON.parse(raw) as { error?: { message?: string } }
        detail = parsed.error?.message ?? raw
      } catch {
        detail = raw
      }
    } catch {
      // keep empty detail
    }
    throw new Error(`HTTP ${res.status}${detail ? ` — ${detail}` : ''}`)
  }

  const reader = res.body?.getReader()
  if (!reader) {
    throw new Error('无法读取响应流')
  }

  const decoder = new TextDecoder('utf-8')
  let buffer = ''
  let fullText = ''

  const processLine = (line: string) => {
    const trimmed = line.trim()
    if (!trimmed.startsWith('data:')) return
    const payload = trimmed.slice(5).trim()
    if (payload === '[DONE]') return
    try {
      const json = JSON.parse(payload) as {
        choices?: Array<{ delta?: { content?: string } }>
      }
      const delta = json.choices?.[0]?.delta?.content
      if (typeof delta === 'string' && delta) {
        fullText += delta
        opts.onDelta(delta)
      }
    } catch {
      // ignore partial SSE frames
    }
  }

  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() ?? ''
    for (const line of lines) processLine(line)
  }
  if (buffer.trim()) processLine(buffer)
  return fullText
}

/* ═══════════════════════════════════════════════
   Tool-call protocol (text-based function calling)
   ═══════════════════════════════════════════════ */

const extractToolCalls = (
  text: string,
): { text: string; calls: Array<{ command: string; arguments: Record<string, unknown> }> } => {
  const calls: Array<{ command: string; arguments: Record<string, unknown> }> = []
  const blockRe = /```tool_call\s*\r?\n?([\s\S]*?)```/g
  const clean = text.replace(blockRe, (_match, body: string) => {
    try {
      const parsed = JSON.parse(body.trim()) as { command?: unknown; arguments?: unknown }
      if (parsed && typeof parsed.command === 'string') {
        calls.push({
          command: parsed.command,
          arguments: (parsed.arguments && typeof parsed.arguments === 'object'
            ? (parsed.arguments as Record<string, unknown>)
            : {}) as Record<string, unknown>,
        })
      }
    } catch {
      // malformed tool block — strip it silently
    }
    return ''
  })
  return { text: clean.trim(), calls }
}

const executeCommand = async (call: {
  command: string
  arguments: Record<string, unknown>
}): Promise<ToolEvent> => {
  const cmd = commands.find((c) => c.name === call.command)
  if (!cmd) {
    return {
      command: call.command,
      arguments: call.arguments,
      result: `未知指令: ${call.command}`,
      ok: false,
    }
  }

  // Validate required args first — a missing value should make the
  // agent ask the user instead of guessing (e.g. paths or hashes).
  const validation = validateToolArgs(cmd.inputSchema, call.arguments)
  if (!validation.ok) {
    return {
      command: cmd.name,
      arguments: call.arguments ?? {},
      result: validation.message,
      ok: false,
    }
  }

  // Risk gate — write/danger operations need explicit user approval.
  const risk = cmd.risk ?? 'read'
  if (risk === 'write' || risk === 'danger') {
    const isDanger = risk === 'danger'
    try {
      await ElMessageBox.confirm(
        isDanger ? t('xianzun.confirmDangerContent', { command: cmd.name, args: JSON.stringify(call.arguments ?? {}) }) : t('xianzun.confirmWriteContent', { command: cmd.name, args: JSON.stringify(call.arguments ?? {}) }),
        isDanger ? t('xianzun.confirmDanger') : t('xianzun.confirmWrite'),
        {
          type: isDanger ? 'error' : 'warning',
          confirmButtonText: t('xianzun.allow'),
          cancelButtonText: t('xianzun.reject'),
          closeOnClickModal: false,
        },
      )
    } catch {
      return {
        command: cmd.name,
        arguments: call.arguments ?? {},
        result: t('xianzun.userRejected'),
        ok: false,
      }
    }
  }

  try {
    const result = await cmd.execute(call.arguments ?? {})
    return { command: cmd.name, arguments: call.arguments ?? {}, result: String(result), ok: true }
  } catch (err) {
    return {
      command: cmd.name,
      arguments: call.arguments ?? {},
      result: `执行失败: ${errorText(err)}`,
      ok: false,
    }
  }
}

/* ═══════════════════════════════════════════════
   Agent turn loop
   ═══════════════════════════════════════════════ */

const buildApiMessages = (): ApiMessage[] => {
  const list: ApiMessage[] = []
  for (const msg of messages.value) {
    if (msg.role === 'user') list.push({ role: 'user', content: msg.content })
    else if (msg.role === 'assistant' && msg.content.trim()) {
      list.push({ role: 'assistant', content: msg.content })
    }
  }
  return list
}

const runAgentTurn = async () => {
  if (isStreaming.value) return

  const apiKey = appSettings.xianzunApiKey.trim()
  if (!apiKey) {
    messages.value.push({
      id: nextId(),
      role: 'error',
      content: t('xianzun.missingKey'),
      createdAt: Date.now(),
    })
    settingsOpen.value = true
    void scrollToBottom()
    return
  }

  isStreaming.value = true
  abortController = new AbortController()
  const signal = abortController.signal

  const assistantMsg: ChatMessage = {
    id: nextId(),
    role: 'assistant',
    content: '',
    streaming: true,
    toolEvents: [],
    createdAt: Date.now(),
  }
  messages.value.push(assistantMsg)
  void scrollToBottom()

  const toolResultQueue: ApiMessage[] = []
  const model = appSettings.xianzunModel.trim() || 'deepseek-chat'

  try {
    let rounds = 0
    for (;;) {
      rounds += 1
      const history: ApiMessage[] = [
        { role: 'system', content: buildSystemPrompt() },
        ...buildApiMessages(),
        ...toolResultQueue,
      ]

      const raw = await streamChatCompletion({
        apiUrl: appSettings.xianzunApiUrl,
        apiKey,
        model,
        messages: history,
        signal,
        onDelta: (delta) => {
          assistantMsg.content += delta
          scrollToBottomIfNear()
        },
      })

      const { text: cleanText, calls } = extractToolCalls(raw)
      assistantMsg.content = cleanText

      if (calls.length === 0 || rounds >= MAX_TOOL_ROUNDS) {
        break
      }

      for (const call of calls) {
        const evt = await executeCommand(call)
        assistantMsg.toolEvents?.push(evt)
        toolResultQueue.push({
          role: 'user',
          content: [
            `[指令执行结果] 指令: ${evt.command}`,
            `参数: ${JSON.stringify(evt.arguments ?? {})}`,
            `结果: ${evt.result}`,
            '',
            '如果任务已完成,请直接给用户最终答复;如果还需要其他操作,可以继续调用指令。',
          ].join('\n'),
        })
      }
    }
  } catch (err) {
    if (isAbortError(err)) {
      assistantMsg.content = (assistantMsg.content ? assistantMsg.content + ' ' : '') + '⏹'
    } else {
      assistantMsg.content = ''
      messages.value.push({
        id: nextId(),
        role: 'error',
        content: `${t('xianzun.errorPrefix')}: ${errorText(err)}`,
        createdAt: Date.now(),
      })
    }
  } finally {
    assistantMsg.streaming = false
    isStreaming.value = false
    abortController = null
    persist()
    void scrollToBottom()
  }
}

const sendMessage = async () => {
  const text = draft.value.trim()
  if (!text || isStreaming.value) return
  draft.value = ''
  messages.value.push({ id: nextId(), role: 'user', content: text, createdAt: Date.now() })
  persist()
  void scrollToBottom()
  await runAgentTurn()
}

const stopStreaming = () => {
  abortController?.abort()
}

const sendSuggestion = (suggestion: string) => {
  draft.value = suggestion
  void sendMessage()
}

const clearChat = async () => {
  if (messages.value.length === 0) return
  try {
    await ElMessageBox.confirm(t('xianzun.clearConfirm'), t('xianzun.clear'), {
      confirmButtonText: t('xianzun.clear'),
      cancelButtonText: t('xianzun.cancel'),
      type: 'warning',
    })
  } catch {
    return
  }
  messages.value = []
  persist()
}

/* ═══════════════════════════════════════════════
   Connection test
   ═══════════════════════════════════════════════ */

const testConnection = async () => {
  testing.value = true
  try {
    const apiKey = appSettings.xianzunApiKey.trim()
    const base = appSettings.xianzunApiUrl.trim().replace(/\/+$/, '')
    if (!apiKey) {
      ElMessage.warning(t('xianzun.missingKey'))
      return
    }
    const res = await fetch(`${base}/models`, {
      headers: { Authorization: `Bearer ${apiKey}` },
      signal: AbortSignal.timeout(15000),
    })
    if (res.ok) {
      ElMessage.success(t('xianzun.connectOk'))
    } else {
      ElMessage.error(t('xianzun.connectFail', { error: `HTTP ${res.status}` }))
    }
  } catch (err) {
    ElMessage.error(t('xianzun.connectFail', { error: errorText(err) }))
  } finally {
    testing.value = false
  }
}

/* ═══════════════════════════════════════════════
   Clipboard / links / markdown helpers
   ═══════════════════════════════════════════════ */

const copyText = async (text: string) => {
  try {
    await writeText(text)
    ElMessage.success(t('xianzun.copied'))
  } catch (err) {
    ElMessage.error(`${t('xianzun.copyFailed')}: ${errorText(err)}`)
  }
}

const onChatContentClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  const copyBtn = target.closest('[data-copy]') as HTMLElement | null
  if (copyBtn) {
    const payload = copyBtn.dataset.copy ?? ''
    if (payload) void copyText(decodeURIComponent(payload))
    return
  }
  const link = target.closest('[data-href]') as HTMLElement | null
  if (link) {
    const href = link.dataset.href ?? ''
    if (href) void openUrl(decodeURIComponent(href))
  }
}

const escapeHtml = (value: string): string =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')

const renderInline = (value: string): string => {
  let out = value
  // inline code (content already escaped — protect it from other transforms)
  out = out.replace(/`([^`]+)`/g, '<code class="xz-inline-code">$1</code>')
  // markdown links
  out = out.replace(
    /\[([^\]]+)\]\(([^)\s]+)\)/g,
    (_m, label: string, href: string) => {
      const decoded = href.replace(/&amp;/g, '&')
      return `<a href="#" class="xz-link" data-href="${encodeURIComponent(decoded)}">${label}</a>`
    },
  )
  // bold / italic / strikethrough
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/(?<!\*)\*([^*\n]+)\*(?!\*)/g, '<em>$1</em>')
  out = out.replace(/~~([^~]+)~~/g, '<del>$1</del>')
  return out
}

const renderMarkdown = (source: string): string => {
  if (!source) return ''
  const copyLabel = t('xianzun.copy')
  const blocks: string[] = []

  // 1. extract fenced code blocks first (raw, unescaped)
  const fenceRe = /```([\w+-]*)[ \t]*\r?\n?([\s\S]*?)```/g
  let text = source.replace(fenceRe, (_m, langRaw: string, bodyRaw: string) => {
    const lang = (langRaw || 'text').trim() || 'text'
    const body = bodyRaw.replace(/\r?\n$/, '')
    blocks.push(
      `<div class="xz-code"><div class="xz-code-head"><span class="xz-code-lang">${escapeHtml(lang)}</span><button type="button" class="xz-copy-btn" data-copy="${encodeURIComponent(body)}">${escapeHtml(copyLabel)}</button></div><pre><code>${escapeHtml(body)}</code></pre></div>`,
    )
    return `\u0000BLOCK${blocks.length - 1}\u0000`
  })

  // 2. escape everything else, then restore code blocks
  text = escapeHtml(text)
  text = text.replace(/\u0000BLOCK(\d+)\u0000/g, (_m, idx: string) => blocks[Number(idx)] ?? '')

  // 3. line-based block parsing
  const lines = text.split('\n')
  const out: string[] = []
  let i = 0

  while (i < lines.length) {
    const line = lines[i]
    const trimmed = line.trim()

    if (!trimmed) {
      i++
      continue
    }

    // blockquote
    if (trimmed.startsWith('>')) {
      const quote: string[] = []
      while (i < lines.length && lines[i].trim().startsWith('>')) {
        quote.push(lines[i].trim().replace(/^>\s?/, ''))
        i++
      }
      out.push(`<blockquote>${quote.map((l) => renderInline(l)).join('<br>')}</blockquote>`)
      continue
    }

    // headings — keep within h2..h4 to match the app's type scale
    const heading = trimmed.match(/^(#{1,4})\s+(.*)$/)
    if (heading) {
      const level = heading[1].length === 1 ? 2 : heading[1].length === 2 ? 3 : 4
      out.push(`<h${level}>${renderInline(heading[2])}</h${level}>`)
      i++
      continue
    }

    // horizontal rule
    if (/^(\s*[-*_]\s*){3,}$/.test(trimmed)) {
      out.push('<hr>')
      i++
      continue
    }

    // unordered list
    if (/^[-*+]\s+/.test(trimmed)) {
      const items: string[] = []
      while (i < lines.length) {
        const current = lines[i].trim()
        if (/^[-*+]\s+/.test(current)) {
          items.push(`<li>${renderInline(current.replace(/^[-*+]\s+/, ''))}</li>`)
          i++
        } else if (/^\s{2,}/.test(lines[i]) && items.length > 0) {
          items[items.length - 1] = items[items.length - 1].replace(
            '</li>',
            `<br>${renderInline(lines[i].trim())}</li>`,
          )
          i++
        } else {
          break
        }
      }
      out.push(`<ul>${items.join('')}</ul>`)
      continue
    }

    // ordered list
    if (/^\d+[.)]\s+/.test(trimmed)) {
      const items: string[] = []
      while (i < lines.length) {
        const current = lines[i].trim()
        if (/^\d+[.)]\s+/.test(current)) {
          items.push(`<li>${renderInline(current.replace(/^\d+[.)]\s+/, ''))}</li>`)
          i++
        } else {
          break
        }
      }
      out.push(`<ol>${items.join('')}</ol>`)
      continue
    }

    // table (header row + separator row, then body rows)
    if (/^\|.*\|$/.test(trimmed) && i + 1 < lines.length) {
      const separator = lines[i + 1].trim()
      if (/^\|?[\s:|-]+\|?$/.test(separator) && separator.includes('-')) {
        const headerCells = trimmed
          .replace(/^\||\|$/g, '')
          .split('|')
          .map((c) => renderInline(c.trim()))
        i += 2
        const rows: string[] = []
        while (i < lines.length && /^\|.*\|$/.test(lines[i].trim())) {
          const cells = lines[i]
            .trim()
            .replace(/^\||\|$/g, '')
            .split('|')
            .map((c) => renderInline(c.trim()))
          rows.push(`<tr>${cells.map((c) => `<td>${c}</td>`).join('')}</tr>`)
          i++
        }
        out.push(
          `<table><thead><tr>${headerCells.map((c) => `<th>${c}</th>`).join('')}</tr></thead>${rows.length > 0 ? `<tbody>${rows.join('')}</tbody>` : ''}</table>`,
        )
        continue
      }
    }

    // paragraph
    const para: string[] = [trimmed]
    i++
    while (i < lines.length && lines[i].trim()) {
      para.push(lines[i].trim())
      i++
    }
    out.push(`<p>${para.map((l) => renderInline(l)).join('<br>')}</p>`)
  }

  return out.join('\n')
}

const renderContent = (msg: ChatMessage) =>
  msg.role === 'assistant' ? renderMarkdown(msg.content) : escapeHtml(msg.content)

/* ═══════════════════════════════════════════════
   Scroll & input behaviors
   ═══════════════════════════════════════════════ */

const scrollToBottom = async () => {
  await nextTick()
  const el = chatListRef.value
  if (el) el.scrollTop = el.scrollHeight
}

const isNearBottom = (): boolean => {
  const el = chatListRef.value
  if (!el) return true
  return el.scrollHeight - el.scrollTop - el.clientHeight < 160
}

const scrollToBottomIfNear = () => {
  if (isNearBottom()) void scrollToBottom()
}

const autoResize = () => {
  const el = inputRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${Math.min(el.scrollHeight, 160)}px`
}

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) {
    event.preventDefault()
    void sendMessage()
  }
}

const onSendClick = () => {
  if (isStreaming.value) {
    stopStreaming()
  } else {
    void sendMessage()
  }
}

const toggleTool = (msgId: string) => {
  const idx = expandedTools.value.indexOf(msgId)
  if (idx >= 0) expandedTools.value.splice(idx, 1)
  else expandedTools.value.push(msgId)
}

const formatTime = (ts: number) => {
  const d = new Date(ts)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`
}

const isToolExpanded = (msgId: string) => expandedTools.value.includes(msgId)

/* ═══════════════════════════════════════════════
   Lifecycle
   ═══════════════════════════════════════════════ */

watch(draft, () => autoResize())
watch(() => messages.value.length, () => void scrollToBottom())

onMounted(() => {
  loadMessages()
  nextTick(() => {
    autoResize()
    void scrollToBottom()
    if (messages.value.length === 0) inputRef.value?.focus()
  })
})
</script>

<template>
  <div class="xz-page">
    <!-- ═══ Header ═══ -->
    <header class="xz-header">
      <div class="xz-brand">
        <div class="xz-avatar" aria-hidden="true">
          <span class="xz-avatar-glyph">尊</span>
          <span class="xz-avatar-glow"></span>
        </div>
        <div class="xz-brand-text">
          <div class="xz-name">
            {{ t('xianzun.nav') }}
            <span class="xz-name-en">XianZun</span>
          </div>
          <div class="xz-status">
            <span class="xz-dot" :class="statusClass"></span>
            {{ statusText }}
          </div>
        </div>
      </div>

      <div class="xz-header-actions">
        <el-select
          v-model="appSettings.xianzunModel"
          class="xz-model-select"
          filterable
          allow-create
          default-first-option
          :placeholder="t('xianzun.model')"
        >
          <el-option label="deepseek-chat" value="deepseek-chat" />
          <el-option label="deepseek-reasoner" value="deepseek-reasoner" />
        </el-select>

        <el-tooltip :content="t('xianzun.settings')" placement="bottom" :show-after="250">
          <button type="button" class="xz-icon-btn" :class="{ active: settingsOpen }" @click="settingsOpen = true">
            <el-icon><Setting /></el-icon>
          </button>
        </el-tooltip>

        <el-tooltip :content="t('xianzun.clear')" placement="bottom" :show-after="250">
          <button type="button" class="xz-icon-btn" @click="clearChat">
            <el-icon><Delete /></el-icon>
          </button>
        </el-tooltip>
      </div>
    </header>

    <!-- ═══ Chat list ═══ -->
    <main ref="chatListRef" class="xz-chat glass-scrollbar">
      <!-- Empty state -->
      <div v-if="messages.length === 0" class="xz-empty">
        <div class="xz-empty-orb" aria-hidden="true">
          <span class="xz-empty-glyph">尊</span>
          <span class="xz-empty-glow"></span>
        </div>
        <h2 class="xz-empty-title">{{ t('xianzun.welcomeTitle') }}</h2>
        <p class="xz-empty-desc">{{ t('xianzun.welcomeDesc') }}</p>

        <div class="xz-suggestions">
          <button
            v-for="suggestion in suggestionList"
            :key="suggestion"
            type="button"
            class="xz-suggestion"
            @click="sendSuggestion(suggestion)"
          >
            <el-icon><MagicStick /></el-icon>
            <span>{{ suggestion }}</span>
          </button>
        </div>

        <div class="xz-capabilities">
          <div class="xz-capabilities-head">
            <el-icon><ChatDotRound /></el-icon>
            <span>{{ t('xianzun.capabilities') }}</span>
            <span class="xz-capabilities-badge">MCP · {{ commands.length }}</span>
          </div>
          <div v-for="group in capabilityGroups" :key="group.key" class="xz-capability-group">
            <div class="xz-capability-group-label">{{ group.label }} · {{ group.tools.length }}</div>
            <div class="xz-capability-chips">
              <span
                v-for="cmd in group.tools"
                :key="cmd.name"
                class="xz-capability-chip"
                :title="cmd.description"
              >
                {{ cmd.name }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Messages -->
      <template v-else>
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="xz-msg"
          :class="[msg.role, { streaming: msg.streaming }]"
        >
          <div v-if="msg.role !== 'user'" class="xz-mini-avatar" aria-hidden="true">尊</div>

          <div class="xz-msg-main">
            <div class="xz-bubble" :class="{ error: msg.role === 'error' }" @click="onChatContentClick">
              <!-- User text is plain; assistant content is markdown -->
              <template v-if="msg.role === 'user' || msg.role === 'error'">
                <div class="xz-plain-text">{{ msg.content }}</div>
              </template>
              <template v-else>
                <div class="xz-markdown" v-html="renderContent(msg)"></div>
                <span v-if="msg.streaming" class="xz-caret" aria-hidden="true"></span>
              </template>

              <!-- Tool call cards -->
              <div v-if="msg.toolEvents && msg.toolEvents.length > 0" class="xz-tools">
                <div
                  v-for="(evt, idx) in msg.toolEvents"
                  :key="idx"
                  class="xz-tool-card"
                  :class="{ ok: evt.ok, fail: !evt.ok }"
                >
                  <button type="button" class="xz-tool-head" @click="toggleTool(msg.id + '-' + idx)">
                    <span class="xz-tool-state">{{ evt.ok ? '✓' : '✕' }}</span>
                    <code class="xz-tool-name">{{ evt.command }}</code>
                    <span class="xz-tool-args">{{ JSON.stringify(evt.arguments ?? {}) }}</span>
                    <span class="xz-tool-chevron">{{ isToolExpanded(msg.id + '-' + idx) ? '▾' : '▸' }}</span>
                  </button>
                  <div v-if="isToolExpanded(msg.id + '-' + idx)" class="xz-tool-body">{{ evt.result }}</div>
                </div>
              </div>
            </div>

            <div class="xz-msg-time">
              {{ formatTime(msg.createdAt) }}
              <button
                v-if="msg.role === 'assistant' && msg.content && !msg.streaming"
                type="button"
                class="xz-copy-row-btn"
                @click="copyText(msg.content)"
              >
                <el-icon><CopyDocument /></el-icon>
                <span>{{ t('xianzun.copy') }}</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Waiting for first token -->
        <div v-if="waitingFirstToken" class="xz-msg assistant">
          <div class="xz-mini-avatar" aria-hidden="true">尊</div>
          <div class="xz-msg-main">
            <div class="xz-bubble">
              <span class="xz-typing" aria-label="thinking">
                <i></i><i></i><i></i>
              </span>
            </div>
          </div>
        </div>
      </template>
    </main>

    <!-- ═══ Composer ═══ -->
    <footer class="xz-composer-wrap">
      <div class="xz-composer" :class="{ streaming: isStreaming }">
        <textarea
          ref="inputRef"
          v-model="draft"
          class="xz-input"
          rows="1"
          :placeholder="t('xianzun.placeholder')"
          @keydown="onKeydown"
          @input="autoResize"
        ></textarea>
        <div class="xz-composer-bar">
          <span class="xz-hint">
            {{ t('xianzun.hint') }}
            <span class="xz-hint-sep">·</span>
            <span class="xz-hint-model">{{ appSettings.xianzunModel || 'deepseek-chat' }}</span>
          </span>
          <button
            type="button"
            class="xz-send"
            :class="{ stop: isStreaming, disabled: !draft.trim() && !isStreaming }"
            :title="isStreaming ? t('xianzun.stop') : t('xianzun.send')"
            @click="onSendClick"
          >
            <el-icon v-if="!isStreaming"><Promotion /></el-icon>
            <el-icon v-else class="xz-stop-icon"><VideoPause /></el-icon>
          </button>
        </div>
      </div>
    </footer>

    <!-- ═══ Settings dialog ═══ -->
    <el-dialog
      v-model="settingsOpen"
      class="glass-dialog xz-settings-dialog"
      :title="t('xianzun.settingsTitle')"
      width="540px"
      align-center
    >
      <div class="xz-settings">
        <label class="xz-field">
          <span class="xz-field-label">{{ t('xianzun.apiKey') }}</span>
          <el-input
            v-model="appSettings.xianzunApiKey"
            type="password"
            show-password
            :placeholder="t('xianzun.apiKeyPlaceholder')"
          />
        </label>

        <label class="xz-field">
          <span class="xz-field-label">{{ t('xianzun.apiUrl') }}</span>
          <el-input v-model="appSettings.xianzunApiUrl" :placeholder="'https://api.deepseek.com/v1'" />
        </label>

        <label class="xz-field">
          <span class="xz-field-label">{{ t('xianzun.model') }}</span>
          <el-select
            v-model="appSettings.xianzunModel"
            filterable
            allow-create
            default-first-option
            class="xz-settings-model"
          >
            <el-option label="deepseek-chat" value="deepseek-chat" />
            <el-option label="deepseek-reasoner" value="deepseek-reasoner" />
          </el-select>
        </label>

        <label class="xz-field">
          <span class="xz-field-label">{{ t('xianzun.systemPrompt') }}</span>
          <el-input
            v-model="appSettings.xianzunSystemPrompt"
            type="textarea"
            :rows="4"
            :placeholder="t('xianzun.systemPromptPlaceholder')"
          />
        </label>

        <p class="xz-settings-note">{{ t('xianzun.settingsNote') }}</p>
      </div>

      <template #footer>
        <el-button :loading="testing" @click="testConnection">
          <el-icon v-if="!testing"><LinkIcon /></el-icon>
          <span>{{ t('xianzun.testConnection') }}</span>
        </el-button>
        <el-button type="primary" @click="settingsOpen = false">
          <span>{{ t('xianzun.done') }}</span>
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════
   Page layout — full-height flex, chat scrolls,
   composer pinned at the bottom.
   ═══════════════════════════════════════════════ */

.xz-page {
  height: 100%;
  min-height: 0;
  width: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  color: rgba(var(--theme-text-primary-rgb), 0.96);
}

/* ── Header ── */
.xz-header {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 8px 12px;
  border-bottom: 1px solid rgba(var(--theme-surface-tint-rgb), 0.08);
}

.xz-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.xz-avatar {
  position: relative;
  flex: 0 0 auto;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(145deg, rgba(var(--theme-surface-tint-rgb), 0.16), rgba(var(--theme-surface-tint-rgb), 0.04));
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.22);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.25);
  overflow: hidden;
}

.xz-avatar-glyph {
  position: relative;
  z-index: 1;
  font-size: 20px;
  font-weight: 700;
  color: rgba(var(--theme-text-primary-rgb), 0.96);
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
}

.xz-avatar-glow {
  position: absolute;
  top: -40%;
  left: -30%;
  width: 120%;
  height: 120%;
  background: radial-gradient(circle at 50% 50%, rgba(var(--theme-surface-tint-rgb), 0.28), transparent 60%);
  pointer-events: none;
}

.xz-brand-text {
  min-width: 0;
}

.xz-name {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 15px;
  line-height: 1.25;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.xz-name-en {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.6px;
  color: rgba(var(--theme-text-secondary-rgb), 0.6);
}

.xz-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 3px;
  font-size: 11.5px;
  color: rgba(var(--theme-text-secondary-rgb), 0.72);
}

.xz-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: 0 0 auto;
}

.xz-dot.online {
  background: var(--theme-success);
  box-shadow: 0 0 8px rgba(var(--theme-success-rgb), 0.6);
}

.xz-dot.offline {
  background: rgba(var(--theme-text-secondary-rgb), 0.45);
}

.xz-dot.streaming {
  background: var(--theme-warning);
  box-shadow: 0 0 8px rgba(var(--theme-warning-rgb), 0.6);
  animation: xz-pulse 1.1s ease-in-out infinite;
}

@keyframes xz-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

.xz-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 0 0 auto;
}

.xz-model-select {
  width: 172px;
}

.xz-model-select :deep(.el-select__wrapper) {
  min-height: 32px;
  border-radius: 8px;
  background: rgba(var(--theme-surface-tint-rgb), 0.06);
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.14);
  box-shadow: none;
  transition: background-color 160ms ease, border-color 160ms ease;
}

.xz-model-select :deep(.el-select__wrapper:hover) {
  background: rgba(var(--theme-surface-tint-rgb), 0.1);
}

.xz-icon-btn {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.14);
  background: rgba(var(--theme-surface-tint-rgb), 0.06);
  color: rgba(var(--theme-text-primary-rgb), 0.85);
  cursor: pointer;
  transition: background-color 160ms ease, border-color 160ms ease, color 160ms ease;
}

.xz-icon-btn:hover,
.xz-icon-btn.active {
  background: rgba(var(--theme-surface-tint-rgb), 0.14);
  border-color: rgba(var(--theme-surface-tint-rgb), 0.28);
  color: rgba(var(--theme-text-primary-rgb), 1);
}

/* ── Chat list ── */
.xz-chat {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 10px 6px 16px;
}

.xz-msg {
  display: flex;
  gap: 10px;
  margin: 16px 0;
  animation: xz-msg-in 0.22s ease-out both;
}

@keyframes xz-msg-in {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.xz-msg.user {
  justify-content: flex-end;
}

.xz-mini-avatar {
  flex: 0 0 auto;
  width: 28px;
  height: 28px;
  margin-top: 2px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  color: rgba(var(--theme-text-primary-rgb), 0.92);
  background: linear-gradient(145deg, rgba(var(--theme-surface-tint-rgb), 0.14), rgba(var(--theme-surface-tint-rgb), 0.04));
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.18);
}

.xz-msg-main {
  max-width: min(78%, 780px);
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.xz-msg.user .xz-msg-main {
  align-items: flex-end;
}

.xz-bubble {
  position: relative;
  padding: 11px 14px;
  border-radius: 16px;
  font-size: 13.5px;
  line-height: 1.75;
  overflow-wrap: break-word;
  word-break: break-word;
}

.xz-bubble.error {
  background: var(--t-danger-bg);
  border: 1px solid var(--t-danger-border);
  color: var(--t-danger-text);
}

.xz-msg.user .xz-bubble {
  background: linear-gradient(145deg, rgba(var(--theme-surface-tint-rgb), 0.15), rgba(var(--theme-surface-tint-rgb), 0.06));
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.2);
  border-bottom-right-radius: 5px;
}

.xz-msg.assistant .xz-bubble,
.xz-msg:not(.user):not(.error) .xz-bubble {
  background: var(--t-material-bg);
  border: var(--t-material-border);
  box-shadow: var(--t-shadow-section);
  border-bottom-left-radius: 5px;
}

.xz-plain-text {
  white-space: pre-wrap;
  user-select: text;
}

.xz-msg-time {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 5px;
  padding: 0 4px;
  font-size: 10.5px;
  color: rgba(var(--theme-text-secondary-rgb), 0.45);
}

.xz-msg.user .xz-msg-time {
  justify-content: flex-end;
}

.xz-copy-row-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 1px 6px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: rgba(var(--theme-text-secondary-rgb), 0.55);
  font-size: 10.5px;
  cursor: pointer;
  transition: background-color 140ms ease, color 140ms ease;
}

.xz-copy-row-btn:hover {
  background: rgba(var(--theme-surface-tint-rgb), 0.1);
  color: rgba(var(--theme-text-primary-rgb), 0.9);
}

/* streaming caret */
.xz-caret {
  display: inline-block;
  width: 2px;
  height: 15px;
  margin-left: 3px;
  vertical-align: -2px;
  background: rgba(var(--theme-surface-tint-rgb), 0.85);
  animation: xz-blink 0.9s step-end infinite;
}

@keyframes xz-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* typing indicator */
.xz-typing {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  padding: 2px 2px;
}

.xz-typing i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(var(--theme-surface-tint-rgb), 0.55);
  animation: xz-bounce 1.2s ease-in-out infinite;
}

.xz-typing i:nth-child(2) { animation-delay: 0.15s; }
.xz-typing i:nth-child(3) { animation-delay: 0.3s; }

@keyframes xz-bounce {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.45; }
  30% { transform: translateY(-4px); opacity: 1; }
}

/* tool call cards */
.xz-tools {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 10px;
  border-top: 1px solid rgba(var(--theme-surface-tint-rgb), 0.08);
  padding-top: 8px;
}

.xz-tool-card {
  border-radius: 10px;
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.12);
  background: rgba(var(--theme-surface-tint-rgb), 0.04);
  overflow: hidden;
}

.xz-tool-card.ok {
  border-left: 3px solid rgba(var(--theme-success-rgb), 0.6);
}

.xz-tool-card.fail {
  border-left: 3px solid rgba(var(--theme-danger-rgb), 0.7);
}

.xz-tool-head {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border: none;
  background: transparent;
  color: rgba(var(--theme-text-secondary-rgb), 0.85);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}

.xz-tool-head:hover {
  background: rgba(var(--theme-surface-tint-rgb), 0.06);
}

.xz-tool-state {
  flex: 0 0 auto;
  font-size: 11px;
}

.xz-tool-card.ok .xz-tool-state { color: var(--t-success-text); }
.xz-tool-card.fail .xz-tool-state { color: var(--t-danger-text); }

.xz-tool-name {
  flex: 0 0 auto;
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 11.5px;
  color: rgba(var(--theme-text-primary-rgb), 0.92);
}

.xz-tool-args {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 11px;
  color: rgba(var(--theme-text-secondary-rgb), 0.55);
}

.xz-tool-chevron {
  flex: 0 0 auto;
  color: rgba(var(--theme-text-secondary-rgb), 0.5);
}

.xz-tool-body {
  padding: 8px 12px;
  border-top: 1px solid rgba(var(--theme-surface-tint-rgb), 0.07);
  font-size: 11.5px;
  line-height: 1.6;
  color: rgba(var(--theme-text-secondary-rgb), 0.8);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 180px;
  overflow-y: auto;
  user-select: text;
}

/* ── Empty state ── */
.xz-empty {
  max-width: 640px;
  margin: 0 auto;
  padding: 26px 12px 30px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.xz-empty-orb {
  position: relative;
  width: 74px;
  height: 74px;
  border-radius: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(150deg, rgba(var(--theme-surface-tint-rgb), 0.2), rgba(var(--theme-surface-tint-rgb), 0.05));
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.26);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.3);
  overflow: hidden;
}

.xz-empty-glyph {
  position: relative;
  z-index: 1;
  font-size: 34px;
  font-weight: 700;
  color: rgba(var(--theme-text-primary-rgb), 0.97);
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.45);
}

.xz-empty-glow {
  position: absolute;
  top: -50%;
  left: -35%;
  width: 150%;
  height: 150%;
  background: radial-gradient(circle at 50% 50%, rgba(var(--theme-surface-tint-rgb), 0.32), transparent 62%);
  pointer-events: none;
}

.xz-empty-title {
  margin: 20px 0 0;
  font-size: 20px;
  line-height: 1.3;
  font-weight: 700;
  letter-spacing: 0.2px;
}

.xz-empty-desc {
  margin: 8px 0 0;
  max-width: 460px;
  font-size: 13px;
  line-height: 1.7;
  color: rgba(var(--theme-text-secondary-rgb), 0.75);
}

.xz-suggestions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 10px;
  margin-top: 22px;
}

.xz-suggestion {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 9px 14px;
  border-radius: 999px;
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.16);
  background: rgba(var(--theme-surface-tint-rgb), 0.07);
  color: rgba(var(--theme-text-primary-rgb), 0.88);
  font-size: 12.5px;
  cursor: pointer;
  transition: background-color 160ms ease, border-color 160ms ease, transform 160ms ease;
}

.xz-suggestion:hover {
  background: rgba(var(--theme-surface-tint-rgb), 0.14);
  border-color: rgba(var(--theme-surface-tint-rgb), 0.3);
  transform: translateY(-1px);
}

.xz-capabilities {
  width: 100%;
  margin-top: 30px;
  text-align: left;
}

.xz-capabilities-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 650;
  color: rgba(var(--theme-text-secondary-rgb), 0.85);
}

.xz-capabilities-badge {
  padding: 1px 7px;
  border-radius: 999px;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.8px;
  color: rgba(var(--theme-success-rgb), 0.9);
  border: 1px solid rgba(var(--theme-success-rgb), 0.35);
  background: rgba(var(--theme-success-rgb), 0.08);
}

.xz-capability-group {
  margin-top: 14px;
}

.xz-capability-group-label {
  font-size: 11px;
  font-weight: 650;
  letter-spacing: 0.4px;
  text-transform: uppercase;
  color: rgba(var(--theme-text-secondary-rgb), 0.6);
}

.xz-capability-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 7px;
  max-height: 170px;
  overflow-y: auto;
  scrollbar-width: thin;
}

.xz-capability-chip {
  padding: 3px 9px;
  border-radius: 999px;
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 10.5px;
  color: rgba(var(--theme-text-secondary-rgb), 0.85);
  background: rgba(var(--theme-surface-tint-rgb), 0.06);
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.1);
  cursor: default;
  transition: background-color 140ms ease, color 140ms ease, border-color 140ms ease;
}

.xz-capability-chip:hover {
  background: rgba(var(--theme-surface-tint-rgb), 0.12);
  border-color: rgba(var(--theme-surface-tint-rgb), 0.22);
  color: rgba(var(--theme-text-primary-rgb), 0.95);
}

/* ── Composer ── */
.xz-composer-wrap {
  flex: 0 0 auto;
  padding: 10px 4px 14px;
}

.xz-composer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px 10px;
  border-radius: 18px;
  background: var(--t-material-bg);
  border: var(--t-material-border);
  box-shadow: var(--t-material-shadow);
  backdrop-filter: var(--t-blur-medium);
  -webkit-backdrop-filter: var(--t-blur-medium);
  transition: border-color 180ms ease, box-shadow 180ms ease;
}

.xz-composer:focus-within {
  border-color: rgba(var(--theme-surface-tint-rgb), 0.3);
  box-shadow: 0 0 0 3px rgba(var(--theme-surface-tint-rgb), 0.06), var(--t-material-shadow);
}

.xz-composer.streaming {
  border-color: rgba(var(--theme-warning-rgb), 0.3);
}

.xz-input {
  width: 100%;
  max-height: 160px;
  padding: 2px 2px 0;
  border: none;
  outline: none;
  resize: none;
  background: transparent;
  color: rgba(var(--theme-text-primary-rgb), 0.95);
  font-family: inherit;
  font-size: 13.5px;
  line-height: 1.6;
  overflow-y: auto;
}

.xz-input::placeholder {
  color: rgba(var(--theme-text-secondary-rgb), 0.45);
}

.xz-composer-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.xz-hint {
  font-size: 11px;
  color: rgba(var(--theme-text-secondary-rgb), 0.5);
  display: flex;
  align-items: center;
  gap: 6px;
}

.xz-hint-sep {
  opacity: 0.6;
}

.xz-hint-model {
  font-family: 'Cascadia Code', Consolas, monospace;
  color: rgba(var(--theme-text-secondary-rgb), 0.7);
}

.xz-send {
  flex: 0 0 auto;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.3);
  background: rgba(var(--theme-surface-tint-rgb), 0.13);
  color: rgba(var(--theme-text-primary-rgb), 0.95);
  cursor: pointer;
  transition: background-color 160ms ease, border-color 160ms ease, transform 120ms ease;
}

.xz-send:hover:not(.disabled) {
  background: rgba(var(--theme-surface-tint-rgb), 0.22);
  border-color: rgba(var(--theme-surface-tint-rgb), 0.45);
}

.xz-send:active:not(.disabled) {
  transform: scale(0.94);
}

.xz-send.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.xz-send.stop {
  border-color: rgba(var(--theme-danger-rgb), 0.45);
  background: rgba(var(--theme-danger-rgb), 0.16);
}

.xz-send.stop:hover {
  background: rgba(var(--theme-danger-rgb), 0.26);
}

.xz-stop-icon {
  font-size: 15px;
}

/* ── Settings dialog ── */
.xz-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.xz-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.xz-field-label {
  font-size: 12.5px;
  font-weight: 600;
  color: rgba(var(--theme-text-primary-rgb), 0.9);
}

.xz-settings-model {
  width: 100%;
}

.xz-settings-note {
  margin: 0;
  font-size: 11.5px;
  line-height: 1.6;
  color: rgba(var(--theme-text-secondary-rgb), 0.55);
}

/* ── Responsive ── */
@media (max-width: 760px) {
  .xz-msg-main {
    max-width: 88%;
  }

  .xz-model-select {
    width: 140px;
  }

  .xz-hint {
    display: none;
  }
}

/* ── Reduced motion ── */
@media (prefers-reduced-motion: reduce) {
  .xz-page *,
  .xz-page *::before,
  .xz-page *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>

<style>
/* ═══════════════════════════════════════════════
   Markdown rendering (unscoped — v-html content)
   ═══════════════════════════════════════════════ */

.xz-markdown {
  user-select: text;
  font-size: 13.5px;
  line-height: 1.75;
}

.xz-markdown > :first-child {
  margin-top: 0;
}

.xz-markdown > :last-child {
  margin-bottom: 0;
}

.xz-markdown p {
  margin: 8px 0;
}

.xz-markdown h2,
.xz-markdown h3,
.xz-markdown h4 {
  margin: 16px 0 8px;
  line-height: 1.35;
  font-weight: 700;
  color: rgba(var(--theme-text-primary-rgb), 0.97);
}

.xz-markdown h2 { font-size: 16px; }
.xz-markdown h3 { font-size: 14.5px; }
.xz-markdown h4 { font-size: 13.5px; }

.xz-markdown ul,
.xz-markdown ol {
  margin: 8px 0;
  padding-left: 22px;
}

.xz-markdown li {
  margin: 3px 0;
}

.xz-markdown blockquote {
  margin: 10px 0;
  padding: 2px 12px;
  border-left: 3px solid rgba(var(--theme-surface-tint-rgb), 0.4);
  color: rgba(var(--theme-text-secondary-rgb), 0.78);
}

.xz-markdown hr {
  margin: 16px 0;
  border: none;
  height: 1px;
  background: rgba(var(--theme-surface-tint-rgb), 0.12);
}

.xz-markdown a.xz-link {
  color: rgba(var(--theme-surface-tint-rgb), 0.92);
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}

.xz-markdown code.xz-inline-code {
  padding: 1px 6px;
  border-radius: 5px;
  background: rgba(var(--theme-surface-tint-rgb), 0.11);
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 0.88em;
  color: rgba(var(--theme-text-primary-rgb), 0.94);
}

.xz-markdown table {
  width: 100%;
  margin: 12px 0;
  border-collapse: collapse;
  font-size: 12.5px;
}

.xz-markdown th,
.xz-markdown td {
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.12);
  padding: 6px 10px;
  text-align: left;
}

.xz-markdown th {
  background: rgba(var(--theme-surface-tint-rgb), 0.07);
  font-weight: 650;
  color: rgba(var(--theme-text-primary-rgb), 0.92);
}

.xz-markdown strong {
  color: rgba(var(--theme-text-primary-rgb), 1);
  font-weight: 700;
}

/* code block */
.xz-code {
  margin: 12px 0;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid rgba(var(--theme-surface-tint-rgb), 0.12);
  background: rgba(0, 0, 0, 0.38);
}

.xz-code-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 4px 6px 4px 12px;
  background: rgba(var(--theme-surface-tint-rgb), 0.05);
  border-bottom: 1px solid rgba(var(--theme-surface-tint-rgb), 0.07);
}

.xz-code-lang {
  font-size: 10.5px;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: rgba(var(--theme-text-secondary-rgb), 0.6);
}

.xz-copy-btn {
  padding: 2px 8px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: rgba(var(--theme-text-secondary-rgb), 0.7);
  font-size: 11px;
  cursor: pointer;
  transition: background-color 140ms ease, color 140ms ease;
}

.xz-copy-btn:hover {
  background: rgba(var(--theme-surface-tint-rgb), 0.12);
  color: rgba(var(--theme-text-primary-rgb), 1);
}

.xz-code pre {
  margin: 0;
  padding: 11px 13px;
  overflow-x: auto;
  scrollbar-width: thin;
}

.xz-code pre::-webkit-scrollbar {
  height: 6px;
}

.xz-code pre::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.18);
  border-radius: 3px;
}

.xz-code code {
  background: transparent;
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.65;
  color: rgba(255, 255, 255, 0.9);
  user-select: text;
}
</style>
