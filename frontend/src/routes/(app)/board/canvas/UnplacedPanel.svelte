<script module lang="ts">
	/** dataTransfer type carrying a node id from the panel onto the canvas. */
	export const UNPLACED_DRAG_TYPE = 'application/x-loom-node';
</script>

<script lang="ts">
	import Badge from '$lib/components/ui/Badge.svelte';
	import { KIND_GLYPH, accentColor } from '$lib/graph/display';
	import { parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * Nodes with no canvas position yet. Drag one onto the canvas (or press
	 * Place for the middle of the view) to put it there.
	 */
	let {
		nodes,
		highlight,
		onplace
	}: { nodes: NodeResponse[]; highlight: string | null; onplace: (nodeId: string) => void } =
		$props();

	let query = $state('');

	const parentOf = $derived(parentPathOf(graph.edges));
	const listed = $derived(
		nodes.filter(
			(node) =>
				matchesBoardFilters(node) && node.title.toLowerCase().includes(query.trim().toLowerCase())
		)
	);

	function toggle() {
		prefs.unplacedPanelOpen = !prefs.unplacedPanelOpen;
		savePrefs();
	}
</script>

<aside class="unplaced" class:expanded={prefs.unplacedPanelOpen}>
	<button type="button" class="tab" aria-expanded={prefs.unplacedPanelOpen} onclick={toggle}>
		Unplaced · {nodes.length}
	</button>
	{#if prefs.unplacedPanelOpen}
		<div class="body">
			<div class="head">
				<span class="label">Unplaced</span>
				<span class="hint">drag onto the canvas</span>
			</div>
			<input class="field" type="search" placeholder="Find a node…" bind:value={query} />
			<ul class="list">
				{#each listed as node (node.id)}
					{@const path = graph.nodeById.get(parentOf.get(node.id) ?? '')}
					<li
						class:highlight={node.id === highlight}
						draggable="true"
						ondragstart={(event) => {
							event.dataTransfer?.setData(UNPLACED_DRAG_TYPE, node.id);
							if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
						}}
					>
						<button type="button" class="row-main" onclick={() => openNode(node.id)}>
							<span class="glyph" style:color={accentColor(node)}>{KIND_GLYPH[node.kind]}</span>
							<span class="title">{node.title}</span>
							{#if path}<span class="in">in {path.title}</span>{/if}
						</button>
						<Badge status={node.status} blocked={node.blocked} />
						<button type="button" class="place" onclick={() => onplace(node.id)}>Place</button>
					</li>
				{:else}
					<li class="none">{nodes.length ? 'No matches.' : 'Everything is on the canvas.'}</li>
				{/each}
			</ul>
		</div>
	{/if}
</aside>

<style>
	.unplaced {
		display: flex;
		background: var(--surface);
		border-left: var(--border-width-hair) solid var(--line);
		min-height: 0;
	}

	.unplaced.expanded {
		width: 33%;
		min-width: 260px;
		max-width: 440px;
	}

	.tab {
		writing-mode: vertical-rl;
		padding: 14px 7px;
		border: none;
		border-right: var(--border-width-hair) solid var(--line);
		background: var(--surface-2);
		font: 700 10px/1 var(--font-mono);
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-2);
		cursor: pointer;
	}

	.tab:hover {
		color: var(--ink);
	}

	.body {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		overflow-y: auto;
	}

	.head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}

	.hint {
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.list li {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		cursor: grab;
	}

	.list li.highlight {
		border-color: var(--accent);
		box-shadow: inset 3px 0 0 var(--accent);
	}

	.row-main {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: baseline;
		gap: 7px;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		color: var(--ink);
	}

	.row-main:hover .title {
		color: var(--accent);
	}

	.title {
		font: 700 13px/1.25 var(--font-display);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.in {
		flex: none;
		max-width: 40%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.place {
		padding: 4px 7px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 600 10px/1 var(--font-mono);
		color: var(--ink-2);
		cursor: pointer;
	}

	.place:hover {
		border-color: var(--ink);
		color: var(--ink);
	}

	.list li.none {
		cursor: default;
		border-style: dashed;
		font: 500 12.5px/1.4 var(--font-display);
		color: var(--ink-2);
	}
</style>
