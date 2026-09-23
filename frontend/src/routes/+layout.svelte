<script lang="ts">
	import '@fontsource/space-grotesk/400.css';
	import '@fontsource/space-grotesk/500.css';
	import '@fontsource/space-grotesk/600.css';
	import '@fontsource/space-grotesk/700.css';
	import '@fontsource/space-mono/400.css';
	import '@fontsource/space-mono/700.css';
	import '$lib/styles/global.css';
	import favicon from '$lib/assets/favicon.svg';
	import { bootstrapSession, session } from '$lib/stores/session.svelte';

	let { children } = $props();

	bootstrapSession();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Loom</title>
</svelte:head>

{#if session.ready}
	{@render children()}
{/if}
