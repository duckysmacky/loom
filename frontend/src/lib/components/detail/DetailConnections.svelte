<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { edgesApi } from '$lib/api/endpoints';
	import { accentColor } from '$lib/graph/display';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let { node }: { node: NodeResponse } = $props();

	// Edges are stored in one direction only; each relation reads it from
	// this node's point of view.
	const RELATIONS = {
		requires: { label: 'Requires', kind: 'requires', outgoing: true },
		required_by: { label: 'Required by', kind: 'requires', outgoing: false },
		part_of: { label: 'Part of', kind: 'part_of', outgoing: true },
		contains: { label: 'Contains', kind: 'part_of', outgoing: false },
		related: { label: 'Related', kind: 'related', outgoing: true }
	} as const;
	type Relation = keyof typeof RELATIONS;

	const connections = $derived(
		graph.edges.flatMap((edge) => {
			const outgoing = edge.from_node_id === node.id;
			if (!outgoing && edge.to_node_id !== node.id) return [];
			const other = graph.nodeById.get(outgoing ? edge.to_node_id : edge.from_node_id);
			if (!other) return [];
			const relation: Relation =
				edge.kind === 'related'
					? 'related'
					: edge.kind === 'requires'
						? outgoing
							? 'requires'
							: 'required_by'
						: outgoing
							? 'part_of'
							: 'contains';
			// Only this node's own requirements can block it.
			const unmet = relation === 'requires' && other.status !== 'done';
			return [{ edge, other, relation, unmet }];
		})
	);

	let adding = $state(false);
	let relation = $state<Relation>('requires');
	let otherId = $state('');

	const candidates = $derived(
		graph.nodes
			.filter((candidate) => candidate.id !== node.id && candidate.status !== 'archived')
			.toSorted((left, right) => left.title.localeCompare(right.title))
	);

	async function add() {
		if (!otherId) return;
		const { kind, outgoing } = RELATIONS[relation];
		const request: CreateEdgeRequest = outgoing
			? { from_node_id: node.id, to_node_id: otherId, kind }
			: { from_node_id: otherId, to_node_id: node.id, kind };
		const created = await graph.mutate(() => edgesApi.create(request));
		if (created) {
			otherId = '';
			adding = false;
		}
	}
</script>

<section>
	<div class="label">Connections</div>
	<ul class="list">
		{#each connections as connection (connection.edge.id)}
			<li class="row">
				<span class="relation" class:unmet={connection.unmet}>
					{RELATIONS[connection.relation].label}
				</span>
				<span class="dot" style:background={accentColor(connection.other)}></span>
				<button type="button" class="title" onclick={() => openNode(connection.other.id)}>
					{connection.other.title}
				</button>
				{#if connection.relation === 'requires'}
					<span class="state" class:unmet={connection.unmet}>
						{connection.unmet ? 'unmet' : 'met ✓'}
					</span>
				{/if}
				<button
					type="button"
					class="remove"
					aria-label="Remove connection to {connection.other.title}"
					onclick={() => graph.mutate(() => edgesApi.remove(connection.edge.id))}>✕</button
				>
			</li>
		{:else}
			<li class="none">No connections</li>
		{/each}
	</ul>

	{#if adding}
		<div class="add">
			<select class="field" aria-label="Relation" bind:value={relation}>
				{#each Object.entries(RELATIONS) as [value, { label }] (value)}
					<option {value}>{label}</option>
				{/each}
			</select>
			<select class="field" aria-label="Node" bind:value={otherId}>
				<option value="">Choose a node…</option>
				{#each candidates as candidate (candidate.id)}
					<option value={candidate.id}>{candidate.title}</option>
				{/each}
			</select>
			<div class="add-actions">
				<Button variant="quiet" onclick={() => (adding = false)}>Cancel</Button>
				<Button variant="primary" onclick={add} disabled={!otherId}>Connect</Button>
			</div>
		</div>
	{:else}
		<button type="button" class="link" onclick={() => (adding = true)}>+ Add connection</button>
	{/if}
</section>

<style>
	.list {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 9px;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.relation {
		width: 86px;
		flex: none;
		font: 700 9.5px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.relation.unmet,
	.state.unmet {
		color: var(--warn);
	}

	.dot {
		width: 7px;
		height: 7px;
		flex: none;
	}

	.title {
		flex: 1;
		min-width: 0;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		font: 700 13.5px/1.2 var(--font-display);
		color: var(--ink);
		overflow-wrap: anywhere;
	}

	.title:hover {
		color: var(--accent);
	}

	.state {
		font: 500 11px/1 var(--font-mono);
		color: var(--ok);
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
		font: 500 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.link {
		margin-top: 10px;
		border: none;
		background: none;
		padding: 0;
		font: 700 11.5px/1 var(--font-display);
		color: var(--accent);
	}

	.add {
		margin-top: 12px;
		display: grid;
		grid-template-columns: 130px 1fr;
		gap: 6px;
	}

	.add .field {
		padding: 7px 9px;
		font-size: 12px;
	}

	.add-actions {
		grid-column: 1 / -1;
		display: flex;
		justify-content: flex-end;
		gap: 6px;
	}
</style>
