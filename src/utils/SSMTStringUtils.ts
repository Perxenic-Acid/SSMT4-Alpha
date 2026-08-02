export class SSMTStringUtils {
	// Get file hash from a migoto dump filename.
	// Priority is '!S!=' -> '!U!=' -> first '=' and then take following 8 chars.
	public static getFileHashFromFileName(input: string): string {
		const takeAfterMarker = (marker: string, len: number): string => {
			const pos = input.indexOf(marker)
			if (pos < 0) {
				return ''
			}

			const start = pos + marker.length
			const end = start + len
			if (end > input.length) {
				return ''
			}

			return input.slice(start, end)
		}

		if (input.includes('!S!=')) {
			return takeAfterMarker('!S!=', 8)
		}

		if (input.includes('!U!=')) {
			return takeAfterMarker('!U!=', 8)
		}

		const equalPos = input.indexOf('=')
		if (equalPos >= 0) {
			const start = equalPos + 1
			const end = start + 8
			if (end <= input.length) {
				return input.slice(start, end)
			}
		}

		return ''
	}

	// Get 16-char PS hash from '-ps=' segment in filename.
	public static getPSHashFromFileName(input: string): string {
		const marker = '-ps='
		const pos = input.indexOf(marker)
		if (pos < 0) {
			return ''
		}

		const start = pos + marker.length
		const end = start + 16
		if (end > input.length) {
			return ''
		}

		return input.slice(start, end)
	}
}
