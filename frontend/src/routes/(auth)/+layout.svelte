<script lang="ts">
	import { goto } from '$app/navigation';
	import { session } from '$lib/stores/session.svelte';

	let { children } = $props();

	$effect(() => {
		if (session.user) goto('/dashboard', { replaceState: true });
	});
</script>

<main class="auth drafting-grid">
	<div class="sheet">
		<div class="brand">
			<span class="mark"></span>
			<span class="wordmark">LOOM</span>
			<span class="tagline">projects · courses · ideas, as one graph</span>
		</div>
		{@render children()}
	</div>
</main>

<style>
	.auth {
		min-height: 100%;
		display: grid;
		place-items: center;
		padding: 24px 16px;
	}

	.sheet {
		width: min(400px, 100%);
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
	}

	.brand {
		display: flex;
		align-items: center;
		gap: 9px;
		flex-wrap: wrap;
		padding: 16px 22px;
		background: var(--side);
		border-bottom: var(--border-width) solid var(--frame);
	}

	.mark {
		width: 15px;
		height: 15px;
		background: var(--accent);
	}

	.wordmark {
		font: 700 17px/1 var(--font-mono);
		letter-spacing: 0.06em;
		color: var(--side-hi);
	}

	.tagline {
		margin-left: auto;
		font: 400 10px/1 var(--font-mono);
		color: var(--side-ink);
	}
</style>
