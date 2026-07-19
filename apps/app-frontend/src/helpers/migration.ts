import { invoke } from '@tauri-apps/api/core'

export type MigrationCandidate = {
	sourcePath: string
	estimatedSizeBytes: number
}

/**
 * Returns a Modrinth App data directory we could migrate from, or null if
 * Meverinth has already been initialized or no Modrinth App install exists.
 */
export async function find_modrinth_install_candidate(): Promise<MigrationCandidate | null> {
	const result = await invoke<MigrationCandidate | null>(
		'plugin:migration|find_modrinth_install_candidate',
	)
	return result ?? null
}

/**
 * Copies the Modrinth App data directory at the given path into Meverinth's
 * data directory. Must be called before {@link initialize_state}.
 */
export async function migrate_from_modrinth(sourcePath: string): Promise<void> {
	await invoke('plugin:migration|migrate_from_modrinth', { sourcePath })
}
