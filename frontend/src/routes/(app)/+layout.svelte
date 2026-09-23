<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import NodeDetail from '$lib/components/detail/NodeDetail.svelte';
	import QuickCapture from '$lib/components/QuickCapture.svelte';
	import Toaster from '$lib/components/Toaster.svelte';
	import { overlays } from '$lib/stores/ui.svelte';
	import { fade } from 'svelte/transition';
	import { ms } from '$lib/motion';
	import { graph } from '$lib/stores/graph.svelte';
	import { session } from '$lib/stores/session.svelte';
	import Sidebar from './Sidebar.svelte';
	import TopBar from './TopBar.svelte';

	let { children } = $props();

	$effect(() => {
		if (!session.user) goto('/login', { replaceState: true });
		else if (!graph.loaded) graph.load();
	});

	function onkeydown(event: KeyboardEvent) {
		if (event.ctrlKey && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			overlays.paletteOpen = !overlays.paletteOpen;
			return;
		}
		// Plain "N" opens quick capture - Ctrl+N is reserved by browsers.
		const target = event.target as HTMLElement;
		const typing = target.isContentEditable || target.matches('input, textarea, select');
		const modified = event.metaKey || event.ctrlKey || event.altKey;
		if (
			event.key.toLowerCase() === 'n' &&
			!typing &&
			!modified &&
			!document.querySelector('dialog[open]')
		) {
			event.preventDefault();
			overlays.captureOpen = true;
		}
	}

	const pageTitles: [string, string][] = [
		['/dashboard', 'Dashboard'],
		['/board', 'Board'],
		['/nodes', 'Nodes'],
		['/settings', 'Settings']
	];
	const section = $derived(page.url.pathname.split('/')[1]);
	const title = $derived(
		pageTitles.find(([prefix]) => page.url.pathname.startsWith(prefix))?.[1] ?? 'Loom'
	);
</script>

<svelte:head><title>{title} · Loom</title></svelte:head>
<svelte:window {onkeydown} />

{#if session.user}
	<div class="shell drafting-grid">
		<Sidebar />
		<div class="main">
			<TopBar {title} />
			<main class="content">
				<!-- A quick fade between pages; board views share one key so
				     switching Organized/Canvas/Timeline doesn't flash the toolbar. -->
				{#key section}
					<div class="page-fade" in:fade={{ duration: ms(160) }}>{@render children()}</div>
				{/key}
			</main>
		</div>
	</div>
	<NodeDetail />
	<QuickCapture />
	<CommandPalette />
	<Toaster />
{/if}

<style>
	.shell {
		display: flex;
		min-height: 100%;
	}

	.main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.content {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}

	.page-fade {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}

	/* Narrow screens: the sidebar folds into a strip above the top bar. */
	@media (max-width: 900px) {
		.shell {
			flex-direction: column;
		}
	}
</style>
