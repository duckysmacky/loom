<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { LogOut } from '@lucide/svelte';
	import './settings.css';
	import Button from '$lib/components/ui/Button.svelte';
	import { errorMessage } from '$lib/i18n';
	import { changePassword, logout, session } from '$lib/stores/session.svelte';
	import { notify } from '$lib/stores/toasts.svelte';

	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let saving = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = '';
		if (newPassword !== confirmPassword) {
			error = m.account_passwords_mismatch();
			return;
		}
		saving = true;
		try {
			await changePassword(currentPassword, newPassword);
			currentPassword = newPassword = confirmPassword = '';
			notify(m.account_password_changed());
		} catch (caught) {
			error = errorMessage(caught);
		} finally {
			saving = false;
		}
	}
</script>

<section class="settings-section">
	<h2>{m.settings_account()}</h2>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.auth_email()}</span>
			<span class="row-help">{m.account_email_help()}</span>
		</span>
		<span class="value">{session.user?.email}</span>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.sign_out()}</span>
			<span class="row-help">{m.account_sign_out_help()}</span>
		</span>
		<Button onclick={logout} icon={LogOut}>{m.sign_out()}</Button>
	</div>
</section>

<section class="settings-section">
	<h2>{m.account_change_password()}</h2>
	<p class="explain">{m.account_change_password_help()}</p>
	<form onsubmit={submit}>
		{#if error}<div class="error" role="alert">{error}</div>{/if}
		<label>
			<span class="label">{m.account_current_password()}</span>
			<input
				class="field"
				type="password"
				autocomplete="current-password"
				required
				bind:value={currentPassword}
			/>
		</label>
		<label>
			<span class="label">{m.account_new_password()}</span>
			<input
				class="field"
				type="password"
				autocomplete="new-password"
				minlength="8"
				maxlength="128"
				required
				bind:value={newPassword}
			/>
		</label>
		<label>
			<span class="label">{m.account_repeat_password()}</span>
			<input
				class="field"
				type="password"
				autocomplete="new-password"
				required
				bind:value={confirmPassword}
			/>
		</label>
		<div>
			<Button variant="primary" type="submit" disabled={saving}
				>{m.account_change_password()}</Button
			>
		</div>
	</form>
</section>

<style>
	.value {
		font: 500 12.5px/1 var(--font-mono);
		color: var(--ink-3);
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 360px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 7px;
	}

	.error {
		background: var(--warn-tint);
		border: var(--border-width-hair) solid var(--warn);
		padding: 9px 11px;
		font: 600 12.5px/1.4 var(--font-display);
	}
</style>
