<template>
	<NewModal ref="modal" header="Migrate from Modrinth App?" :closable="!busy">
		<div class="flex w-[30rem] max-w-full flex-col gap-4">
			<p class="m-0 leading-snug">
				We found an existing
				<span class="font-semibold text-contrast">Modrinth App</span> install on this machine.
				Meverinth can copy everything from it — your instances, accounts, settings, skins, and
				caches — so you can pick up where you left off.
			</p>
			<div class="rounded-xl border border-solid border-surface-5 bg-bg-raised p-3 text-sm">
				<div class="flex justify-between gap-4">
					<span class="text-secondary">Source folder</span>
					<span class="truncate font-mono text-contrast" :title="candidate.sourcePath">
						{{ candidate.sourcePath }}
					</span>
				</div>
				<div class="mt-1 flex justify-between gap-4">
					<span class="text-secondary">Approximate size</span>
					<span class="text-contrast">{{ formattedSize }}</span>
				</div>
			</div>
			<p class="m-0 text-xs leading-snug text-secondary">
				The original Modrinth App data is left untouched — Meverinth only copies the files, so
				you can keep using both apps side by side. If you decide later you don't need the
				original install, you can delete that folder yourself.
			</p>

			<p v-if="errorMessage" class="m-0 text-sm font-medium text-red">
				{{ errorMessage }}
			</p>

			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button :disabled="busy" @click="onSkip"><XIcon /> Start fresh</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="busy" @click="onMigrate">
						<SpinnerIcon v-if="busy" class="animate-spin" />
						<DownloadIcon v-else />
						{{ busy ? 'Copying data…' : 'Migrate' }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal, useFormatBytes } from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import type { MigrationCandidate } from '@/helpers/migration'
import { migrate_from_modrinth } from '@/helpers/migration'

const props = defineProps<{
	candidate: MigrationCandidate
}>()

const emit = defineEmits<{
	(e: 'done', migrated: boolean): void
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const busy = ref(false)
const errorMessage = ref<string | null>(null)

const formatBytes = useFormatBytes()
const formattedSize = computed(() =>
	props.candidate.estimatedSizeBytes > 0
		? formatBytes(props.candidate.estimatedSizeBytes)
		: 'Unknown',
)

function show() {
	errorMessage.value = null
	busy.value = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function onMigrate() {
	busy.value = true
	errorMessage.value = null
	try {
		await migrate_from_modrinth(props.candidate.sourcePath)
		hide()
		emit('done', true)
	} catch (error) {
		console.error('Migration failed', error)
		const message =
			error && typeof error === 'object' && 'message' in error
				? String((error as { message: unknown }).message)
				: String(error)
		errorMessage.value = `Migration failed: ${message}`
		busy.value = false
	}
}

function onSkip() {
	hide()
	emit('done', false)
}

defineExpose({ show, hide })
</script>
