<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Check, X } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Button from '$lib/components/ui/Button.svelte';
	import { oauthApi } from '$lib/api/endpoints';
	import { around, errorMessage } from '$lib/i18n';
	import { session } from '$lib/stores/session.svelte';
	import type { AuthorizeDecision } from '$lib/types/AuthorizeDecision';
	import type { AuthorizePreview } from '$lib/types/AuthorizePreview';

	let preview = $state<AuthorizePreview | null>(null);
	let error = $state('');
	let deciding = $state(false);

	$effect(() => {
		if (!session.user) {
			const here = `${page.url.pathname}${page.url.search}`;
			goto(`/login?next=${encodeURIComponent(here)}`, { replaceState: true });
			return;
		}
		oauthApi.preview(page.url.search).then(
			(result) => (preview = result),
			(caught) => (error = errorMessage(caught))
		);
	});

	async function decide(approve: boolean) {
		deciding = true;
		try {
			const request = Object.fromEntries(page.url.searchParams);
			const { redirect_to } = await oauthApi.decide({
				...request,
				approve
			} as AuthorizeDecision);
			// Only ever a redirect URI the client registered - the backend
			// checked it (https or loopback) before minting this.
			window.location.href = redirect_to;
		} catch (caught) {
			error = errorMessage(caught);
			deciding = false;
		}
	}
</script>

<svelte:head><title>{m.oauth_title()} · Loom</title></svelte:head>

<div class="consent">
	<div class="label">{m.oauth_title()}</div>

	{#if error}
		<div class="error" role="alert">
			{m.oauth_error_retry({ error })}
		</div>
	{:else if preview}
		{@const [hostBefore, hostAfter] = around(
			m.oauth_redirect({ host: preview.redirect_host }),
			preview.redirect_host
		)}
		{@const [before, after] = around(
			m.oauth_wants_access({ client: preview.client_name }),
			preview.client_name
		)}
		<p class="lead">{before}<strong>{preview.client_name}</strong>{after}</p>
		<p class="detail">
			{m.oauth_scope()}
			{hostBefore}<strong>{preview.redirect_host}</strong>{hostAfter}
		</p>
		<p class="detail">{m.oauth_disconnect_hint()}</p>
		<div class="actions">
			<Button variant="quiet" disabled={deciding} onclick={() => decide(false)} icon={X}
				>{m.oauth_deny()}</Button
			>
			<Button variant="primary" disabled={deciding} onclick={() => decide(true)} icon={Check}
				>{m.oauth_approve()}</Button
			>
		</div>
	{:else}
		<p class="detail">{m.oauth_checking()}</p>
	{/if}
</div>

<style>
	.consent {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 22px;
	}

	.lead {
		margin: 0;
		font: 500 15px/1.4 var(--font-display);
	}

	.detail {
		margin: 0;
		font: 500 12.5px/1.5 var(--font-display);
		color: var(--ink-2);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
		padding-top: 4px;
	}

	.error {
		background: var(--warn-tint);
		border: var(--border-width-hair) solid var(--warn);
		color: var(--ink);
		padding: 9px 11px;
		font: 600 12.5px/1.4 var(--font-display);
	}
</style>
