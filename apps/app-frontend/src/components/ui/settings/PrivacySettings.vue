<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'

const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>

<template>
	<div class="rounded-xl border border-solid border-surface-5 bg-bg-raised p-4">
		<h2 class="m-0 text-lg font-semibold text-contrast">Privacy-focused build</h2>
		<p class="m-0 mt-1 text-sm">
			Meverinth is a privacy-focused fork of the Modrinth desktop app. Telemetry, error
			reporting, the in-app support chat, user surveys, ads, and download/playtime analytics
			have been removed. No usage data is sent to Modrinth and no third-party trackers are
			loaded. Account sign-in and content downloads still use the public Modrinth API; nothing
			else does.
		</p>
	</div>

	<div class="mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">Discord RPC</h2>
			<p class="m-0 mt-1 text-sm">
				Manages the Discord Rich Presence integration. Disabling this will cause 'Modrinth' to no
				longer show up as a game or app you are using on your Discord profile.
			</p>
			<p class="m-0 mt-2 text-sm">
				Note: This will not prevent any instance-specific Discord Rich Presence integrations, such
				as those added by mods. (app restart required to take effect)
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>
