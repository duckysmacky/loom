<script lang="ts">
	import { goto } from '$app/navigation';
	import Button from '$lib/components/ui/Button.svelte';
	import { ApiError } from '$lib/api/client';

	let {
		heading,
		submitLabel,
		passwordAutocomplete,
		onsubmit,
		footer
	}: {
		heading: string;
		submitLabel: string;
		passwordAutocomplete: 'current-password' | 'new-password';
		onsubmit: (email: string, password: string) => Promise<void>;
		footer: import('svelte').Snippet;
	} = $props();

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let submitting = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = '';
		submitting = true;
		try {
			await onsubmit(email, password);
			await goto('/dashboard');
		} catch (caught) {
			error = caught instanceof ApiError ? caught.message : 'Could not reach the server';
		} finally {
			submitting = false;
		}
	}
</script>

<form class="form" onsubmit={submit}>
	<div class="label">{heading}</div>

	{#if error}<div class="error" role="alert">{error}</div>{/if}

	<label>
		<span class="label">Email</span>
		<input class="field" type="email" autocomplete="email" required bind:value={email} />
	</label>
	<label>
		<span class="label">Password</span>
		<input
			class="field"
			type="password"
			autocomplete={passwordAutocomplete}
			minlength={passwordAutocomplete === 'new-password' ? 8 : undefined}
			required
			bind:value={password}
		/>
	</label>

	<Button variant="primary" type="submit" disabled={submitting}>{submitLabel}</Button>

	<div class="footer">{@render footer()}</div>
</form>

<style>
	.form {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 22px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 7px;
	}

	.error {
		background: var(--warn-tint);
		border: var(--border-width-hair) solid var(--warn);
		color: var(--ink);
		padding: 9px 11px;
		font: 600 12.5px/1.4 var(--font-display);
	}

	.footer {
		font: 600 12px/1.4 var(--font-display);
		color: var(--ink-2);
	}
</style>
