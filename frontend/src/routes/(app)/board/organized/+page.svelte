<script lang="ts">
	import { flip } from 'svelte/animate';
	import NodeCard from '$lib/components/NodeCard.svelte';
	import { TIER_COLOR } from '$lib/graph/display';
	import { dependencyOrder } from '$lib/graph/order';
	import { matchesOrContainsMatch, parentPathOf } from '$lib/graph/paths';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { NodeStatus } from '$lib/types/NodeStatus';
	import PathBox from './PathBox.svelte';

	const STATUS_ORDER: NodeStatus[] = ['active', 'queued', 'paused', 'idea', 'done', 'archived'];
	const byStatus = (left: NodeResponse, right: NodeResponse) =>
		STATUS_ORDER.indexOf(left.status) - STATUS_ORDER.indexOf(right.status);

	// Status order first, then dependency order on top: prerequisites before
	// what they unblock, so each tier reads left to right as a sequence.
	const ordered = $derived(dependencyOrder(graph.nodes.toSorted(byStatus), graph.edges));
	const parentOf = $derived(parentPathOf(graph.edges));

	// A path stays visible while anything inside it matches the filters.
	const isShown = (node: NodeResponse) =>
		matchesOrContainsMatch(node, ordered, parentOf, matchesBoardFilters);

	const childrenOf = (pathId: string) =>
		ordered.filter((node) => parentOf.get(node.id) === pathId && isShown(node));

	// Nodes inside a path render only inside its box; paths sit among the
	// other cards of their tier.
	const topLevel = $derived(ordered.filter((node) => !parentOf.has(node.id) && isShown(node)));
	const tier = (focus: NodeResponse['focus']) => topLevel.filter((node) => node.focus === focus);

	const sections = $derived([
		{ id: 'primary', title: 'Primary', bar: TIER_COLOR.primary, nodes: tier('primary') },
		{ id: 'secondary', title: 'Secondary', bar: TIER_COLOR.secondary, nodes: tier('secondary') },
		{ id: 'background', title: 'Background', bar: TIER_COLOR.background, nodes: tier('background') }
	]);
</script>

<div class="page">
	{#if !graph.loaded}
		<p class="muted">Loading…</p>
	{:else if !topLevel.length}
		<div class="empty">
			{graph.nodes.length
				? 'No nodes match these filters.'
				: 'No nodes yet. Press N to capture one.'}
		</div>
	{/if}

	{#each sections as section (section.id)}
		{#if section.nodes.length}
			<section aria-labelledby="tier-{section.id}">
				<div class="section-head">
					<span class="bar" style:background={section.bar}></span>
					<h2 class="title" id="tier-{section.id}">{section.title}</h2>
					<span class="meta">
						{section.nodes.length}
						{section.nodes.length === 1 ? 'node' : 'nodes'}
					</span>
				</div>
				<div class="row">
					{#each section.nodes as node (node.id)}
						<div
							class={node.kind === 'path' ? 'path-slot' : 'card-slot'}
							animate:flip={{ duration: 200 }}
						>
							{#if node.kind === 'path'}
								<PathBox path={node} {childrenOf} />
							{:else}
								<NodeCard {node} />
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{/each}
</div>

<style>
	.page {
		/* One card footprint for the whole view, path contents included. */
		--card-w: 260px;
		--card-h: 212px;
		padding: var(--page-pad);
		display: flex;
		flex-direction: column;
		gap: 22px;
	}

	h2 {
		margin: 0;
	}

	/* Cards and path boxes flow side by side; a path box is as wide as its
	   contents, so rows mix fixed-size cards and content-sized boxes. */
	.row {
		margin-top: 12px;
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		align-items: flex-start;
	}

	.empty {
		border: var(--border-width-hair) dashed var(--line);
		background: var(--surface);
		padding: 24px;
		text-align: center;
		color: var(--ink-2);
	}

	.card-slot {
		width: var(--card-w);
		height: var(--card-h);
	}

	.path-slot {
		max-width: 100%;
	}
</style>
