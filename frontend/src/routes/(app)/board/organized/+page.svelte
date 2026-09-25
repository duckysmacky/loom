<script lang="ts">
	import { flip } from 'svelte/animate';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import NodeCard from '$lib/components/NodeCard.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { kindChangeLosses } from '$lib/graph/display';
	import { groupNodes, type Section } from '$lib/graph/grouping';
	import { dependencyOrder, manualOrder } from '$lib/graph/order';
	import { matchesOrContainsMatch, parentPathOf } from '$lib/graph/paths';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { NodeStatus } from '$lib/types/NodeStatus';
	import {
		cancelKindDrop,
		confirmKindDrop,
		dragOverSlot,
		dragState,
		dropAtEnd,
		dropOnCard,
		endDrag,
		resetOrder,
		startDrag,
		type DropZone
	} from './drag.svelte';
	import PathBox from './PathBox.svelte';

	const STATUS_ORDER: NodeStatus[] = ['active', 'queued', 'paused', 'idea', 'done', 'archived'];
	const byStatus = (left: NodeResponse, right: NodeResponse) =>
		STATUS_ORDER.indexOf(left.status) - STATUS_ORDER.indexOf(right.status);

	// Status order first, then dependency order on top: prerequisites before
	// what they unblock. Manual drag order wins over both, on top: ranked
	// nodes sort by rank, unranked nodes keep that auto order after them.
	const ordered = $derived(
		manualOrder(dependencyOrder(graph.nodes.toSorted(byStatus), graph.edges))
	);
	const anyManuallyOrdered = $derived(graph.nodes.some((node) => node.sort_order !== null));
	const parentOf = $derived(parentPathOf(graph.edges));

	// A path stays visible while anything inside it matches the filters.
	const isShown = (node: NodeResponse) =>
		matchesOrContainsMatch(node, ordered, parentOf, matchesBoardFilters);

	const childrenOf = (pathId: string) =>
		ordered.filter((node) => parentOf.get(node.id) === pathId && isShown(node));

	// Nodes inside a path render only inside its box; top-level nodes (paths
	// included) are grouped into sections by the chosen attribute.
	const topLevel = $derived(ordered.filter((node) => !parentOf.has(node.id) && isShown(node)));
	const sections = $derived(groupNodes(topLevel, prefs.organizedGrouping, graph.topics));

	const sectionZone = (section: Section): DropZone => ({
		kind: 'section',
		grouping: prefs.organizedGrouping,
		sectionId: section.id
	});
</script>

<div class="page">
	<div class="group-by">
		<span class="label">Group by</span>
		<SegmentedControl
			label="Group by"
			value={prefs.organizedGrouping}
			onchange={(grouping) => {
				prefs.organizedGrouping = grouping;
				savePrefs();
			}}
			options={[
				{ value: 'focus', label: 'Focus' },
				{ value: 'kind', label: 'Kind' },
				{ value: 'status', label: 'Status' },
				{ value: 'tag', label: 'Tag' }
			]}
		/>
		{#if anyManuallyOrdered}
			<Button variant="quiet" onclick={resetOrder}>Reset order</Button>
		{/if}
	</div>

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
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="row"
					ondragover={(event) => dragState.draggedId && event.preventDefault()}
					ondrop={(event) => {
						event.preventDefault();
						dropAtEnd(
							sectionZone(section),
							section.nodes.map((node) => node.id)
						);
					}}
				>
					{#each section.nodes as node (node.id)}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class={node.kind === 'path' ? 'path-slot' : 'card-slot'}
							class:dragging={dragState.draggedId === node.id}
							class:drop-before={dragState.overId === node.id && dragState.overBefore}
							class:drop-after={dragState.overId === node.id && !dragState.overBefore}
							draggable="true"
							ondragstart={() =>
								startDrag(node.id, prefs.organizedGrouping === 'tag' ? section.id : null)}
							ondragover={(event) => dragOverSlot(node.id, event)}
							ondrop={(event) => {
								event.preventDefault();
								event.stopPropagation();
								dropOnCard(
									node,
									sectionZone(section),
									section.nodes.map((n) => n.id)
								);
							}}
							ondragend={endDrag}
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

<Modal
	open={dragState.pendingKindDrop !== null}
	onclose={cancelKindDrop}
	label="Confirm kind change"
	width={440}
>
	{#if dragState.pendingKindDrop}
		<div class="confirm">
			<div class="label">Change kind to {dragState.pendingKindDrop.kind}?</div>
			<p>Each kind keeps different data. Dropping it here will remove:</p>
			<ul>
				{#each kindChangeLosses(dragState.pendingKindDrop.request.node, dragState.pendingKindDrop.kind) as loss (loss)}
					<li>{loss}</li>
				{/each}
			</ul>
			<div class="confirm-actions">
				<Button variant="quiet" onclick={cancelKindDrop}>Cancel</Button>
				<Button variant="primary" onclick={confirmKindDrop}>Change kind</Button>
			</div>
		</div>
	{/if}
</Modal>

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

	.group-by {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: -8px;
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

	.card-slot,
	.path-slot {
		cursor: grab;
	}

	.dragging {
		opacity: 0.4;
	}

	.drop-before {
		box-shadow: -3px 0 0 var(--accent);
	}

	.drop-after {
		box-shadow: 3px 0 0 var(--accent);
	}

	.confirm {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		font: 500 13.5px/1.5 var(--font-display);
	}

	.confirm p,
	.confirm ul {
		margin: 0;
	}

	.confirm li {
		color: var(--warn);
		font-weight: 700;
	}

	.confirm-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 12px;
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>
