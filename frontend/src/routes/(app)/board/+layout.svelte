<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import FilterBar from './FilterBar.svelte';

	let { children } = $props();

	type BoardView = 'organized' | 'canvas' | 'timeline';
	const current = $derived(page.url.pathname.split('/')[2] as BoardView);
</script>

<div class="toolbar">
	<SegmentedControl
		label="Board view"
		value={current}
		onchange={(view) => goto(`/board/${view}`)}
		options={[
			{ value: 'organized', label: 'Organized' },
			{ value: 'canvas', label: 'Canvas' },
			{ value: 'timeline', label: 'Timeline' }
		]}
	/>
	<FilterBar />
</div>

{@render children()}

<style>
	.toolbar {
		display: flex;
		align-items: center;
		gap: 14px;
		flex-wrap: wrap;
		padding: 11px 22px;
		background: var(--surface);
		border-bottom: var(--border-width-hair) solid var(--line);
		position: relative;
		z-index: 10;
	}
</style>
