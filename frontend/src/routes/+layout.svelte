<script lang="ts">
	import '@fontsource/space-grotesk/400.css';
	import '@fontsource/space-grotesk/500.css';
	import '@fontsource/space-grotesk/600.css';
	import '@fontsource/space-grotesk/700.css';
	import '@fontsource/ibm-plex-mono/400.css';
	import '@fontsource/ibm-plex-mono/500.css';
	import '@fontsource/ibm-plex-mono/600.css';
	import '@fontsource/ibm-plex-mono/700.css';
	import '@fontsource/jetbrains-mono/400.css';
	import '$lib/styles/global.css';
	import favicon from '$lib/assets/favicon.svg';
	import { applyAppearance } from '$lib/stores/prefs.svelte';
	import { bootstrapSession, session } from '$lib/stores/session.svelte';

	let { children } = $props();

	applyAppearance();
	bootstrapSession();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Loom</title>
</svelte:head>

{#if session.ready}
	{@render children()}
{/if}
