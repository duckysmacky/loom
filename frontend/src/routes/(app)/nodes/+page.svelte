<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import PromoteDialog from '$lib/components/PromoteDialog.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { accentColor, isBacklog, notesExcerpt, shortDate } from '$lib/graph/display';
	import { openNode, withParam } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { EdgeKind } from '$lib/types/EdgeKind';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { NodeStatus } from '$lib/types/NodeStatus';
	import type { NodeView } from '$lib/types/NodeView';

	const VIEWS: NodeView[] = ['backlog', 'all', 'archived'];
	const view = $derived<NodeView>(
		VIEWS.find((candidate) => candidate === page.url.searchParams.get('tab')) ?? 'backlog'
	);

	let kind = $state<NodeKind | ''>('');
	let status = $state<NodeStatus | ''>('');
	let focus = $state<NodeFocus | ''>('');
	let topicId = $state('');
	let sort = $state<'captured' | 'title' | 'touched'>('captured');

	let rows = $state<NodeResponse[] | null>(null);
	let promoting = $state<NodeResponse | null>(null);

	// Kind/status/focus/view filter server-side; refetch on any change and
	// whenever the graph cache reloads after a mutation.
	$effect(() => {
		void graph.version;
		// Filters hidden on the current tab don't apply (backlog fixes kind and
		// status; archived fixes status).
		const query = {
			view,
			kind: view === 'backlog' ? null : kind || null,
			status: view === 'all' ? status || null : null,
			focus: focus || null
		};
		nodesApi
			.list(query)
			.then((nodes) => (rows = nodes))
			.catch(notifyError);
	});

	const tabs = $derived([
		{
			value: 'backlog' as const,
			label: 'Backlog',
			count: graph.nodes.filter(isBacklog).length
		},
		{ value: 'all' as const, label: 'All', count: graph.nodes.length },
		{
			value: 'archived' as const,
			label: 'Archived',
			count: graph.nodes.filter((node) => node.status === 'archived').length
		}
	]);

	const visibleRows = $derived.by(() => {
		const filtered = (rows ?? []).filter((node) => !topicId || node.topic_ids.includes(topicId));
		const touched = (node: NodeResponse) => node.last_poked_at ?? '';
		return filtered.toSorted((left, right) =>
			sort === 'title'
				? left.title.localeCompare(right.title)
				: sort === 'touched'
					? touched(right).localeCompare(touched(left))
					: right.created_at.localeCompare(left.created_at)
		);
	});

	const LINK_MARKS: Record<EdgeKind, string> = { requires: '⟵', part_of: '↳', related: '┈' };

	function linksOf(node: NodeResponse) {
		return graph.edges
			.filter((edge) => edge.from_node_id === node.id)
			.flatMap((edge) => {
				const target = graph.nodeById.get(edge.to_node_id);
				if (!target) return [];
				const unmet = edge.kind === 'requires' && target.status !== 'done';
				return [{ id: edge.id, mark: LINK_MARKS[edge.kind], title: target.title, unmet }];
			});
	}

	async function restore(node: NodeResponse) {
		const restored = await graph.mutate(() => nodesApi.update(node.id, { status: 'queued' }));
		if (restored) notify(`Restored “${node.title}” to queued`);
	}

	const footer = $derived(
		view === 'backlog'
			? `${visibleRows.length} ideas · promoting sets a kind, status and focus tier`
			: view === 'archived'
				? `${visibleRows.length} archived · restoring moves a node back to queued`
				: `${visibleRows.length} nodes`
	);
</script>

<Tabs
	{tabs}
	value={view}
	onchange={(tab) => goto(withParam('tab', tab), { noScroll: true, keepFocus: true })}
/>

<div class="page">
	<div class="filters">
		{#if view !== 'backlog'}
			<label class="filter">
				<span>Kind</span>
				<select bind:value={kind}>
					<option value="">Any</option>
					<option value="idea">Idea</option>
					<option value="project">Project</option>
					<option value="course">Course</option>
					<option value="path">Path</option>
				</select>
			</label>
			{#if view === 'all'}
				<label class="filter">
					<span>Status</span>
					<select bind:value={status}>
						<option value="">Any</option>
						{#each ['idea', 'queued', 'active', 'paused', 'done', 'archived'] as option (option)}
							<option value={option}>{option[0].toUpperCase() + option.slice(1)}</option>
						{/each}
					</select>
				</label>
			{/if}
		{/if}
		<label class="filter">
			<span>Focus</span>
			<select bind:value={focus}>
				<option value="">Any</option>
				<option value="primary">Primary</option>
				<option value="secondary">Secondary</option>
				<option value="background">Background</option>
			</select>
		</label>
		<label class="filter">
			<span>Tag</span>
			<select bind:value={topicId}>
				<option value="">Any</option>
				{#each graph.topics as topic (topic.id)}
					<option value={topic.id}>{topic.name}</option>
				{/each}
			</select>
		</label>
		<label class="filter sort">
			<span>Sort</span>
			<select bind:value={sort}>
				<option value="captured">Captured ↓</option>
				<option value="touched">Last touched ↓</option>
				<option value="title">Title A–Z</option>
			</select>
		</label>
	</div>

	<div class="table-wrap">
		<table>
			<thead>
				<tr>
					<th class="dot-col"><span class="visually-hidden">Accent</span></th>
					<th>Node</th>
					<th class="links-col">Links</th>
					<th class="tags-col">Tags</th>
					<th class="date-col">Captured</th>
					<th class="action-col">Action</th>
				</tr>
			</thead>
			<tbody>
				{#if rows === null}
					<tr><td colspan="6" class="empty">Loading…</td></tr>
				{:else if !visibleRows.length}
					<tr>
						<td colspan="6" class="empty">
							{view === 'backlog'
								? 'No unpromoted ideas. Press N to capture one.'
								: view === 'archived'
									? 'Nothing archived.'
									: 'No nodes match these filters.'}
						</td>
					</tr>
				{/if}
				{#each visibleRows as node (node.id)}
					{@const links = linksOf(node)}
					<tr onclick={() => openNode(node.id)}>
						<td class="dot-col">
							<span class="dot" style:background={accentColor(node)}></span>
						</td>
						<td>
							<!-- Focusable handle for keyboard users; its click bubbles to the row. -->
							<button type="button" class="title">
								{node.title}
							</button>
							<div class="sub">
								{#if view !== 'backlog'}
									<span class="kind">
										{node.kind}{node.status === node.kind ? '' : ` · ${node.status}`}
									</span>
								{/if}
								{notesExcerpt(node.notes)}
							</div>
						</td>
						<td class="links-col">
							{#each links.slice(0, 2) as link (link.id)}
								<div class="link" class:unmet={link.unmet}>{link.mark} {link.title}</div>
							{:else}
								<span class="link">no links</span>
							{/each}
							{#if links.length > 2}<div class="link">+{links.length - 2} more</div>{/if}
						</td>
						<td class="tags-col">
							<div class="tags">
								{#each node.topic_ids as id (id)}
									{@const topic = graph.topicById.get(id)}
									{#if topic}<span class="tag">{topic.name}</span>{/if}
								{/each}
							</div>
						</td>
						<td class="date-col">{shortDate(node.created_at)}</td>
						<td class="action-col">
							<!-- stopPropagation: the row itself opens the detail panel. -->
							<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
							<span onclick={(event) => event.stopPropagation()}>
								{#if isBacklog(node)}
									<Button variant="poke" onclick={() => (promoting = node)}>Promote</Button>
								{:else if node.status === 'archived'}
									<Button variant="poke" onclick={() => restore(node)}>Restore</Button>
								{/if}
							</span>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<div class="footer">{footer}</div>
</div>

<PromoteDialog node={promoting} onclose={() => (promoting = null)} />

<style>
	.page {
		padding: 16px 22px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.filters {
		display: flex;
		align-items: center;
		gap: 9px;
		flex-wrap: wrap;
	}

	.filter {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: var(--surface);
		border: var(--border-width-hair) solid var(--line);
		padding: 0 0 0 12px;
		font: 600 12.5px/1 var(--font-display);
		color: var(--ink-3);
	}

	.filter span::after {
		content: ':';
	}

	.filter select {
		border: none;
		background: transparent;
		padding: 8px 10px 8px 2px;
		font: 700 12.5px/1 var(--font-display);
		color: var(--ink);
		cursor: pointer;
	}

	.sort {
		margin-left: auto;
	}

	.table-wrap {
		background: var(--surface);
		border: var(--border-width) solid var(--line);
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		min-width: 640px;
	}

	th {
		text-align: left;
		padding: 10px 14px;
		background: var(--surface-2);
		border-bottom: var(--border-width-hair) solid var(--line);
		font: 700 10.5px/1 var(--font-mono);
		letter-spacing: 0.11em;
		text-transform: uppercase;
		color: var(--ink-2);
		white-space: nowrap;
	}

	td {
		padding: var(--row-pad);
		padding-left: 14px;
		padding-right: 14px;
		border-bottom: var(--border-width-hair) solid var(--line);
		vertical-align: middle;
	}

	tbody tr {
		cursor: pointer;
	}

	tbody tr:hover {
		background: var(--surface-2);
	}

	tbody tr:last-child td {
		border-bottom: none;
	}

	.dot-col {
		width: 16px;
		padding-right: 0;
	}

	.dot {
		display: block;
		width: 7px;
		height: 7px;
		margin: auto;
	}

	.title {
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		font: 700 14.5px/1.2 var(--font-display);
		color: var(--ink);
	}

	.sub {
		margin-top: 4px;
		font: 500 12px/1.3 var(--font-display);
		color: var(--ink-2);
	}

	.kind {
		font: 500 10px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		margin-right: 6px;
	}

	.links-col {
		width: 190px;
	}

	.link {
		font: 500 11.5px/1.4 var(--font-mono);
		color: var(--ink-2);
	}

	.link.unmet {
		color: var(--warn);
	}

	.tags-col {
		width: 130px;
	}

	.tags {
		display: flex;
		gap: 5px;
		flex-wrap: wrap;
	}

	.tag {
		font: 600 10.5px/1 var(--font-display);
		color: var(--ink-3);
		background: var(--chip);
		padding: 5px 8px;
	}

	.date-col {
		width: 90px;
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
		white-space: nowrap;
	}

	.action-col {
		width: 100px;
		text-align: right;
	}

	.empty {
		padding: 28px 18px;
		text-align: center;
		color: var(--ink-2);
		cursor: default;
	}

	.footer {
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
	}
</style>
