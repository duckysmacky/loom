<script lang="ts">
	import { m } from '$lib/paraglide/messages';
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
		label={m.board_view()}
		value={current}
		onchange={(view) => goto(`/board/${view}`)}
		options={[
			{ value: 'organized', label: m.nav_organized() },
			{ value: 'canvas', label: m.nav_canvas() },
			{ value: 'timeline', label: m.nav_timeline() }
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
