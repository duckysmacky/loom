<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Toaster from '$lib/components/Toaster.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { session } from '$lib/stores/session.svelte';
	import Sidebar from './Sidebar.svelte';
	import TopBar from './TopBar.svelte';

	let { children } = $props();

	$effect(() => {
		if (!session.user) goto('/login', { replaceState: true });
		else if (!graph.loaded) graph.load();
	});

	const pageTitles: [string, string][] = [
		['/dashboard', 'Dashboard'],
		['/board', 'Board'],
		['/nodes', 'Nodes'],
		['/settings', 'Settings']
	];
	const title = $derived(
		pageTitles.find(([prefix]) => page.url.pathname.startsWith(prefix))?.[1] ?? 'Loom'
	);
</script>

<svelte:head><title>{title} · Loom</title></svelte:head>

{#if session.user}
	<div class="shell drafting-grid">
		<Sidebar />
		<div class="main">
			<TopBar {title} />
			<main class="content">{@render children()}</main>
		</div>
	</div>
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
</style>
