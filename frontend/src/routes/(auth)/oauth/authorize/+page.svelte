<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Button from '$lib/components/ui/Button.svelte';
	import { ApiError } from '$lib/api/client';
	import { oauthApi } from '$lib/api/endpoints';
	import { session } from '$lib/stores/session.svelte';
	import type { AuthorizeDecision } from '$lib/types/AuthorizeDecision';
	import type { AuthorizePreview } from '$lib/types/AuthorizePreview';

	let preview = $state<AuthorizePreview | null>(null);
	let error = $state('');
	let deciding = $state(false);

	function describe(caught: unknown): string {
		return caught instanceof ApiError ? caught.message : 'Could not reach the server';
	}

	$effect(() => {
		if (!session.user) {
			const here = `${page.url.pathname}${page.url.search}`;
			goto(`/login?next=${encodeURIComponent(here)}`, { replaceState: true });
			return;
		}
		oauthApi.preview(page.url.search).then(
			(result) => (preview = result),
			(caught) => (error = describe(caught))
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
			error = describe(caught);
			deciding = false;
		}
	}
</script>

<svelte:head><title>Connect an app · Loom</title></svelte:head>

<div class="consent">
	<div class="label">Connect an app</div>

	{#if error}
		<div class="error" role="alert">
			{error}. Start the connection again from the app you were connecting.
		</div>
	{:else if preview}
		<p class="lead"><strong>{preview.client_name}</strong> wants to access your Loom.</p>
		<p class="detail">
			It will be able to read, create, change and delete your nodes, edges, checklists and topics.
			After approving you'll be sent back to <strong>{preview.redirect_host}</strong>.
		</p>
		<p class="detail">You can disconnect it any time in Settings → Connections.</p>
		<div class="actions">
			<Button variant="quiet" disabled={deciding} onclick={() => decide(false)}>Deny</Button>
			<Button variant="primary" disabled={deciding} onclick={() => decide(true)}>Approve</Button>
		</div>
	{:else}
		<p class="detail">Checking the request…</p>
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
