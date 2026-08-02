const escapeHtml = (value: string): string => value
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;')

const isSafeUrl = (url: string): boolean => {
  try {
    const parsed = new URL(url)
    return parsed.protocol === 'https:' || parsed.protocol === 'http:'
  } catch {
    return false
  }
}

const sanitizeImageHtml = (rawTag: string): string => {
  const src = rawTag.match(/\ssrc\s*=\s*(["'])(.*?)\1/i)?.[2]?.trim() || ''
  if (!src || !isSafeUrl(src)) {
    return ''
  }

  const alt = rawTag.match(/\salt\s*=\s*(["'])(.*?)\1/i)?.[2]?.trim() || ''
  const width = rawTag.match(/\swidth\s*=\s*(["']?)(\d{1,5})\1/i)?.[2]
  const height = rawTag.match(/\sheight\s*=\s*(["']?)(\d{1,5})\1/i)?.[2]
  const dimensionAttrs = [
    width ? ` width="${escapeHtml(width)}"` : '',
    height ? ` height="${escapeHtml(height)}"` : '',
  ].join('')

  return `<img src="${escapeHtml(src)}" alt="${escapeHtml(alt)}"${dimensionAttrs} loading="lazy" decoding="async">`
}

const renderInlineMarkdown = (rawText: string): string => {
  const tokens: string[] = []
  const stash = (html: string): string => {
    const index = tokens.push(html) - 1
    return `\uE000${index}\uE001`
  }

  let text = rawText
    .replace(/<img\b[^>]*>/gi, (tag) => stash(sanitizeImageHtml(tag)))
    .replace(/!\[([^\]]*)\]\((https?:\/\/[^\s)]+)(?:\s+"[^"]*")?\)/gi, (_match, alt, src) => {
      if (!isSafeUrl(src)) {
        return ''
      }
      return stash(`<img src="${escapeHtml(src)}" alt="${escapeHtml(alt || '')}" loading="lazy" decoding="async">`)
    })
    .replace(/`([^`]+)`/g, (_match, code) => stash(`<code>${escapeHtml(code)}</code>`))
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/gi, (_match, label, href) => {
      if (!isSafeUrl(href)) {
        return escapeHtml(label)
      }
      return stash(`<a href="${escapeHtml(href)}" target="_blank" rel="noreferrer noopener">${escapeHtml(label)}</a>`)
    })

  text = escapeHtml(text)
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/__([^_]+)__/g, '<strong>$1</strong>')

  return text.replace(/\uE000(\d+)\uE001/g, (_match, index) => tokens[Number(index)] || '')
}

export const renderReleaseNotesMarkdown = (markdown: string): string => {
  const lines = (markdown || '').replace(/\r\n/g, '\n').split('\n')
  const html: string[] = []
  let listType: 'ul' | 'ol' | null = null

  const closeList = () => {
    if (listType) {
      html.push(`</${listType}>`)
      listType = null
    }
  }

  for (const line of lines) {
    const trimmed = line.trim()

    if (!trimmed) {
      closeList()
      continue
    }

    const heading = trimmed.match(/^(#{1,3})\s+(.+)$/)
    if (heading) {
      closeList()
      const level = heading[1].length + 2
      html.push(`<h${level}>${renderInlineMarkdown(heading[2])}</h${level}>`)
      continue
    }

    const unordered = line.match(/^\s*[-*+]\s+(.+)$/)
    if (unordered) {
      if (listType !== 'ul') {
        closeList()
        html.push('<ul>')
        listType = 'ul'
      }
      html.push(`<li>${renderInlineMarkdown(unordered[1])}</li>`)
      continue
    }

    const ordered = line.match(/^\s*\d+[.)]\s+(.+)$/)
    if (ordered) {
      if (listType !== 'ol') {
        closeList()
        html.push('<ol>')
        listType = 'ol'
      }
      html.push(`<li>${renderInlineMarkdown(ordered[1])}</li>`)
      continue
    }

    closeList()
    html.push(`<p>${renderInlineMarkdown(trimmed)}</p>`)
  }

  closeList()
  return html.join('')
}
