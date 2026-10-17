<script lang="ts">
	import { edgesApi } from '$lib/api/endpoints';
	import { accentColor, progressText } from '$lib/graph/display';
	import { ancestorPaths, parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { isPathCollapsed, prefs, savePrefs, togglePathCollapsed } from '$lib/stores/prefs.svelte';
	import CollapseToggle from '$lib/components/ui/CollapseToggle.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import PathCanvas from './PathCanvas.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/** A path's contents: the nodes sitting inside it (only paths contain nodes). */
	let { node }: { node: NodeResponse } = $props();

	const parentOf = $derived(parentPathOf(graph.edges));
	const inside = (pathId: string) =>
		graph.edges
			.filter((edge) => edge.kind === 'part_of' && edge.to_node_id === pathId)
			.flatMap((edge) => {
				const child = graph.nodeById.get(edge.from_node_id);
				return child ? [{ edge, child }] : [];
			});
	// Anything not already in a path, except this path and the paths around it.
	const candidates = $derived(
		graph.nodes
			.filter((candidate) => candidate.id !== node.id && !parentOf.has(candidate.id))
			.filter((candidate) => !ancestorPaths(node.id, parentOf).includes(candidate.id))
			.filter((candidate) => !candidate.archived)
			.toSorted((left, right) => left.title.localeCompare(right.title))
	);

	function add(childId: string) {
		if (!childId) return;
		graph.mutate(() =>
			edgesApi.create({ from_node_id: childId, to_node_id: node.id, kind: 'part_of' })
		);
	}
</script>

{#snippet rows(pathId: string, depth: number)}
	{#each inside(pathId) as { edge, child } (edge.id)}
		{@const collapsed = isPathCollapsed('detail', child.id)}
		<li class="row" class:done={child.status === 'done'} style:padding-left="{depth * 18}px">
			{#if child.kind === 'path'}
				<CollapseToggle {collapsed} ontoggle={() => togglePathCollapsed('detail', child.id)} />
			{/if}
			<span class="dot" style:background={accentColor(child)}></span>
			<button type="button" class="title" onclick={() => openNode(child.id)}>
				{child.title}
			</button>
			<span class="kind">{child.kind}{child.status ? ` · ${child.status}` : ''}</span>
			<button
				type="button"
				class="remove"
				aria-label="Take “{child.title}” out of its path"
				onclick={() => graph.mutate(() => edgesApi.remove(edge.id))}>✕</button
			>
		</li>
		{#if child.kind === 'path' && !collapsed}
			{@render rows(child.id, depth + 1)}
		{/if}
	{:else}
		{#if depth === 0}<li class="none">Nothing inside yet.</li>{/if}
	{/each}
{/snippet}

<section>
	<div class="head">
		<span class="label">Contains</span>
		<SegmentedControl
			label="How to show what's inside"
			options={[
				{ value: 'list', label: 'List' },
				{ value: 'canvas', label: 'Canvas' }
			]}
			value={prefs.pathDetailView}
			onchange={(view) => {
				prefs.pathDetailView = view;
				savePrefs();
			}}
		/>
		{#if progressText(node)}
			<span class="count">{progressText(node)}</span>
		{/if}
	</div>
	{#if prefs.pathDetailView === 'canvas'}
		{#key node.id}<PathCanvas path={node} />{/key}
	{:else}
		<ul class="list">
			{@render rows(node.id, 0)}
		</ul>
	{/if}
	{#if candidates.length}
		<select
			class="field add"
			aria-label="Add a node to this path"
			value=""
			onchange={(event) => {
				add(event.currentTarget.value);
				event.currentTarget.value = '';
			}}
		>
			<option value="">+ Add a node to this path…</option>
			{#each candidates as candidate (candidate.id)}
				<option value={candidate.id}>{candidate.title} ({candidate.kind})</option>
			{/each}
		</select>
	{/if}
</section>

<style>
	.head {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.head :global(.segmented button) {
		padding: 5px 10px;
		font-size: 11px;
	}

	.count {
		margin-left: auto;
		font: 600 12px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.list {
		list-style: none;
		margin: 10px 0 0;
		padding: 10px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		background: var(--path-fill);
		border: var(--border-width-hair) dashed var(--node-path);
	}

	.row {
		display: flex;
		align-items: center;
		gap: 9px;
	}

	.dot {
		width: 8px;
		height: 8px;
		flex: none;
	}

	.title {
		flex: 1;
		min-width: 0;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		font: 700 14px/1.25 var(--font-display);
		color: var(--ink);
		overflow-wrap: anywhere;
	}

	.title:hover {
		color: var(--accent);
	}

	.done .title {
		text-decoration: line-through;
		color: var(--ink-2);
	}

	.kind {
		font: 500 10px/1 var(--font-mono);
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.remove {
		border: none;
		background: none;
		padding: 2px 4px;
		font-size: 10px;
		color: var(--ink-2);
	}

	.remove:hover {
		color: var(--warn);
	}

	.none {
		font: 500 12.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	.add {
		margin-top: 8px;
		padding: 8px 10px;
		font-size: 13px;
	}
</style>
