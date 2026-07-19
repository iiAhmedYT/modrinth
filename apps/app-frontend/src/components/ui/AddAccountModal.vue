<template>
	<NewModal ref="modal" header="Add a Minecraft account" :on-hide="reset">
		<div class="flex w-[28rem] max-w-full flex-col gap-4">
			<p v-if="mode === 'choose'" class="m-0 text-secondary leading-snug">
				Choose how you'd like to sign in. Microsoft accounts use the standard Mojang login flow.
				Offline accounts skip authentication and let you pick any username — they only work on
				servers that allow offline players.
			</p>
			<div v-if="mode === 'choose'" class="flex flex-col gap-3">
				<button class="account-option" :disabled="busy" @click="chooseMicrosoft">
					<LogInIcon class="h-6 w-6 shrink-0 text-brand" />
					<div class="flex flex-col items-start gap-1">
						<span class="font-semibold text-contrast">Microsoft account</span>
						<span class="text-sm text-secondary">
							Sign in with your real Minecraft (Mojang/Microsoft) account.
						</span>
					</div>
				</button>
				<button class="account-option" :disabled="busy" @click="mode = 'offline'">
					<UserIcon class="h-6 w-6 shrink-0 text-primary" />
					<div class="flex flex-col items-start gap-1">
						<span class="font-semibold text-contrast">Offline account</span>
						<span class="text-sm text-secondary">
							Use a local username. Works for LAN, single-player, and cracked / offline-mode
							servers.
						</span>
					</div>
				</button>
			</div>

			<form v-else class="flex flex-col gap-4" @submit.prevent="submitOffline">
				<label class="flex flex-col gap-2">
					<span class="text-sm font-semibold text-contrast">Username</span>
					<StyledInput
						v-model="username"
						:icon="UserIcon"
						placeholder="Steve"
						:maxlength="16"
						autocomplete="off"
						:error="!!errorMessage"
						:disabled="busy"
					/>
				</label>
				<p class="m-0 text-xs leading-snug text-secondary">
					1&ndash;16 characters. Letters, digits, and underscores only. The account's UUID is
					derived from this name the same way vanilla Minecraft does.
				</p>
				<p v-if="errorMessage" class="m-0 text-sm font-medium text-red">{{ errorMessage }}</p>
				<div class="flex justify-end gap-2">
					<ButtonStyled type="outlined">
						<button type="button" :disabled="busy" @click="mode = 'choose'">
							<LeftArrowIcon /> Back
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button type="submit" :disabled="busy || !username.trim()">
							<SpinnerIcon v-if="busy" class="animate-spin" />
							<PlusIcon v-else />
							Add account
						</button>
					</ButtonStyled>
				</div>
			</form>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import {
	LeftArrowIcon,
	LogInIcon,
	PlusIcon,
	SpinnerIcon,
	UserIcon,
} from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, NewModal, StyledInput } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { login as login_flow, login_offline } from '@/helpers/auth'
import { handleSevereError } from '@/store/error.js'

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
}

const emit = defineEmits<{
	(e: 'added', credential: MinecraftCredential): void
}>()

const { handleError } = injectNotificationManager()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const mode = ref<'choose' | 'offline'>('choose')
const username = ref('')
const busy = ref(false)
const errorMessage = ref<string | null>(null)

const USERNAME_PATTERN = /^[A-Za-z0-9_]{1,16}$/

function show() {
	reset()
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function reset() {
	mode.value = 'choose'
	username.value = ''
	errorMessage.value = null
	busy.value = false
}

async function chooseMicrosoft() {
	busy.value = true
	try {
		const credential = (await login_flow().catch(handleSevereError)) as
			| MinecraftCredential
			| null
			| undefined
		if (credential) {
			emit('added', credential)
			hide()
		}
	} finally {
		busy.value = false
	}
}

async function submitOffline() {
	const value = username.value.trim()
	if (!USERNAME_PATTERN.test(value)) {
		errorMessage.value =
			'Usernames must be 1-16 characters and use only letters, digits, or underscores.'
		return
	}

	busy.value = true
	errorMessage.value = null
	try {
		const credential = (await login_offline(value).catch((error) => {
			handleError(error)
			return null
		})) as MinecraftCredential | null

		if (credential) {
			emit('added', credential)
			hide()
		}
	} finally {
		busy.value = false
	}
}

defineExpose({ show, hide })
</script>

<style scoped>
.account-option {
	display: flex;
	gap: 0.75rem;
	align-items: center;
	padding: 1rem;
	border-radius: 0.75rem;
	background-color: var(--color-button-bg);
	border: 1px solid var(--color-divider);
	cursor: pointer;
	text-align: left;
	transition: background-color 0.15s ease;
}

.account-option:hover:not(:disabled) {
	background-color: var(--color-button-bg-hover);
}

.account-option:disabled {
	opacity: 0.6;
	cursor: not-allowed;
}
</style>
