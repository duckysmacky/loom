<script lang="ts">
	import { SvelteFlowProvider } from '@xyflow/svelte';

	let { children } = $props();
</script>

<!-- Lets the page itself call useSvelteFlow() (drop position from the panel). -->
<SvelteFlowProvider>{@render children()}</SvelteFlowProvider>
