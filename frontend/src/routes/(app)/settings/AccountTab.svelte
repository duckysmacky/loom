<script lang="ts">
	import './settings.css';
	import Button from '$lib/components/ui/Button.svelte';
	import { ApiError } from '$lib/api/client';
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
			error = 'The new passwords do not match';
			return;
		}
		saving = true;
		try {
			await changePassword(currentPassword, newPassword);
			currentPassword = newPassword = confirmPassword = '';
			notify('Password changed · other sessions signed out');
		} catch (caught) {
			error = caught instanceof ApiError ? caught.message : 'Could not reach the server';
		} finally {
			saving = false;
		}
	}
</script>

<section class="settings-section">
	<h2>Account</h2>
	<div class="settings-row">
		<span>
			<span class="row-label">Email</span>
			<span class="row-help">Single-user instance - this is the only account.</span>
		</span>
		<span class="value">{session.user?.email}</span>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">Sign out</span>
			<span class="row-help">Ends this browser's session.</span>
		</span>
		<Button onclick={logout}>Sign out</Button>
	</div>
</section>

<section class="settings-section">
	<h2>Change password</h2>
	<p class="explain">Every other signed-in browser is signed out; this one stays signed in.</p>
	<form onsubmit={submit}>
		{#if error}<div class="error" role="alert">{error}</div>{/if}
		<label>
			<span class="label">Current password</span>
			<input
				class="field"
				type="password"
				autocomplete="current-password"
				required
				bind:value={currentPassword}
			/>
		</label>
		<label>
			<span class="label">New password</span>
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
			<span class="label">Repeat new password</span>
			<input
				class="field"
				type="password"
				autocomplete="new-password"
				required
				bind:value={confirmPassword}
			/>
		</label>
		<div>
			<Button variant="primary" type="submit" disabled={saving}>Change password</Button>
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
