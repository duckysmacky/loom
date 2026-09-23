<script lang="ts">
	import { edgesApi } from '$lib/api/endpoints';
	import { accentColor } from '$lib/graph/display';
	import { ancestorPaths, parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/** A path's contents: the nodes sitting inside it (only paths contain nodes). */
	let { node }: { node: NodeResponse } = $props();

	const parentOf = $derived(parentPathOf(graph.edges));
	const inside = $derived(
		graph.edges
			.filter((edge) => edge.kind === 'part_of' && edge.to_node_id === node.id)
			.flatMap((edge) => {
				const child = graph.nodeById.get(edge.from_node_id);
				return child ? [{ edge, child }] : [];
			})
	);
	// Anything not already in a path, except this path and the paths around it.
	const candidates = $derived(
		graph.nodes
			.filter((candidate) => candidate.id !== node.id && !parentOf.has(candidate.id))
			.filter((candidate) => !ancestorPaths(node.id, parentOf).includes(candidate.id))
			.filter((candidate) => candidate.status !== 'archived')
			.toSorted((left, right) => left.title.localeCompare(right.title))
	);

	function add(childId: string) {
		if (!childId) return;
		graph.mutate(() =>
			edgesApi.create({ from_node_id: childId, to_node_id: node.id, kind: 'part_of' })
		);
	}
</script>

<section>
	<div class="head">
		<span class="label">Contains</span>
		{#if inside.length}
			<span class="count">
				{inside.filter(({ child }) => child.status === 'done').length} / {inside.length} done
			</span>
		{/if}
	</div>
	<ul class="list">
		{#each inside as { edge, child } (edge.id)}
			<li class="row" class:done={child.status === 'done'}>
				<span class="dot" style:background={accentColor(child)}></span>
				<button type="button" class="title" onclick={() => openNode(child.id)}>
					{child.title}
				</button>
				<span class="kind">{child.kind} · {child.status}</span>
				<button
					type="button"
					class="remove"
					aria-label="Take “{child.title}” out of this path"
					onclick={() => graph.mutate(() => edgesApi.remove(edge.id))}>✕</button
				>
			</li>
		{:else}
			<li class="none">Nothing inside yet.</li>
		{/each}
	</ul>
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
