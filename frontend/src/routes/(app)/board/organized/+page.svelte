<script lang="ts">
	import NodeCard from '$lib/components/NodeCard.svelte';
	import { TIER_COLOR, childrenOf } from '$lib/graph/display';
	import { dependencyOrder } from '$lib/graph/order';
	import { openNode } from '$lib/navigation';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { NodeStatus } from '$lib/types/NodeStatus';

	const STATUS_ORDER: NodeStatus[] = ['active', 'queued', 'paused', 'idea', 'done', 'archived'];
	const byStatus = (left: NodeResponse, right: NodeResponse) =>
		STATUS_ORDER.indexOf(left.status) - STATUS_ORDER.indexOf(right.status);

	// Status order first, then dependency order on top: prerequisites before
	// what they unblock, so each tier reads left to right as a sequence.
	const visible = $derived(
		dependencyOrder(graph.nodes.filter(matchesBoardFilters).toSorted(byStatus), graph.edges)
	);

	// Paths get their own section instead of appearing in a focus tier. Any
	// card with part_of children (path or not) shows them as a checklist.
	const sections = $derived([
		{
			id: 'primary',
			title: 'Primary',
			bar: TIER_COLOR.primary,
			nodes: visible.filter((node) => node.kind !== 'path' && node.focus === 'primary')
		},
		{
			id: 'secondary',
			title: 'Secondary',
			bar: TIER_COLOR.secondary,
			nodes: visible.filter((node) => node.kind !== 'path' && node.focus === 'secondary')
		},
		{
			id: 'background',
			title: 'Background',
			bar: TIER_COLOR.background,
			nodes: visible.filter((node) => node.kind !== 'path' && node.focus === 'background')
		},
		{
			id: 'paths',
			title: 'Paths',
			bar: 'var(--node-path)',
			nodes: visible.filter((node) => node.kind === 'path')
		}
	]);
</script>

<div class="page">
	{#if !graph.loaded}
		<p class="muted">Loading…</p>
	{:else if !visible.length}
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
						{section.id === 'paths' ? 'in flight' : section.nodes.length === 1 ? 'node' : 'nodes'}
					</span>
				</div>
				<div class="grid">
					{#each section.nodes as node (node.id)}
						{#if node.container_progress}
							<NodeCard {node}>
								<ul class="checklist">
									{#each childrenOf(node.id, graph.edges, graph.nodeById) as child (child.id)}
										<li>
											<button
												type="button"
												class="child"
												class:done={child.status === 'done'}
												onclick={(event) => {
													event.stopPropagation();
													openNode(child.id);
												}}
											>
												{#if child.status === 'done'}
													<span class="check">✓</span>
												{:else}
													<span class="circle"></span>
												{/if}
												<span class="child-title">{child.title}</span>
											</button>
										</li>
									{/each}
								</ul>
							</NodeCard>
						{:else}
							<NodeCard {node} />
						{/if}
					{/each}
				</div>
			</section>
		{/if}
	{/each}
</div>

<style>
	.page {
		padding: var(--page-pad);
		display: flex;
		flex-direction: column;
		gap: 22px;
	}

	h2 {
		margin: 0;
	}

	.grid {
		margin-top: 12px;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		/* Every card in a section gets the same width and height. */
		grid-auto-rows: 1fr;
		gap: 12px;
		align-items: stretch;
	}

	.empty {
		border: var(--border-width-hair) dashed var(--line);
		background: var(--surface);
		padding: 24px;
		text-align: center;
		color: var(--ink-2);
	}

	.checklist {
		list-style: none;
		margin: 11px 0 0;
		padding: 9px;
		border: var(--border-width-hair) dashed var(--line);
		display: flex;
		flex-direction: column;
		gap: 7px;
	}

	.child {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		font: 600 12.5px/1.35 var(--font-display);
		color: var(--ink);
		min-height: 20px;
	}

	.child:hover .child-title {
		color: var(--accent);
	}

	.child.done .child-title {
		text-decoration: line-through;
		color: var(--ink-2);
	}

	.check {
		width: 12px;
		color: var(--ok);
		font-weight: 700;
	}

	.circle {
		width: 12px;
		height: 12px;
		flex: none;
		border: 1.5px solid var(--line);
		border-radius: 50%;
	}
</style>
