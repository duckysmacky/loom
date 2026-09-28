<script lang="ts">
	import NodeCard from '$lib/components/NodeCard.svelte';
	import AnimatedNumber from '$lib/components/ui/AnimatedNumber.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import CollapseToggle from '$lib/components/ui/CollapseToggle.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { dashboardApi, nodesApi } from '$lib/api/endpoints';
	import {
		TIER_COLOR,
		accentColor,
		canPoke,
		progressPair,
		progressText,
		relativeDays,
		shortDate
	} from '$lib/graph/display';
	import { parentPathOf } from '$lib/graph/paths';
	import { graph } from '$lib/stores/graph.svelte';
	import { isPathCollapsed, togglePathCollapsed } from '$lib/stores/prefs.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import { openNode } from '$lib/navigation';
	import type { DashboardResponse } from '$lib/types/DashboardResponse';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let dashboard = $state<DashboardResponse | null>(null);

	// Paths nest: list the outermost ones, and the rest inside them.
	const parentOf = $derived(parentPathOf(graph.edges));
	const topPaths = $derived.by(() => {
		const listed = new Set(dashboard?.paths.map((path) => path.id));
		return dashboard?.paths.filter((path) => !listed.has(parentOf.get(path.id) ?? '')) ?? [];
	});
	const insideOf = (pathId: string) =>
		graph.nodes.filter((node) => parentOf.get(node.id) === pathId && node.kind !== 'idea');
	let loadFailed = $state(false);
	let ideaTitle = $state('');

	// Refetch whenever the graph cache reloads, i.e. after any mutation.
	$effect(() => {
		void graph.version;
		dashboardApi
			.get()
			.then((response) => {
				dashboard = response;
				loadFailed = false;
			})
			.catch((error) => {
				loadFailed = true;
				notifyError(error);
			});
	});

	const actionableCount = $derived(dashboard?.active.filter((node) => !node.blocked).length ?? 0);
	// The active list arrives ordered by tier; split it into one group per tier.
	const activeTiers = $derived(
		(['primary', 'secondary', 'background'] as const)
			.map((tier) => ({
				tier,
				nodes: dashboard?.active.filter((node) => node.focus === tier) ?? []
			}))
			.filter((group) => group.nodes.length)
	);
	const blockedActiveCount = $derived(dashboard?.active.filter((node) => node.blocked).length ?? 0);

	const tiles = $derived(
		dashboard
			? [
					{ label: 'Total nodes', value: dashboard.counts.total, tone: '' },
					{ label: 'Active', value: dashboard.counts.by_status.active, tone: '' },
					{ label: 'Blocked', value: dashboard.counts.blocked, tone: 'warn' },
					{ label: 'Ideas', value: dashboard.counts.ideas, tone: '' },
					{ label: 'Done', value: dashboard.counts.by_status.done, tone: 'ok' }
				]
			: []
	);

	async function captureIdea(event: SubmitEvent) {
		event.preventDefault();
		const title = ideaTitle.trim();
		if (!title) return;
		const created = await graph.mutate(() => nodesApi.create({ kind: 'idea', title }));
		if (created) {
			ideaTitle = '';
			notify(`Captured “${created.title}”`);
		}
	}

	function poke(node: NodeResponse) {
		graph.poke(node).then((poked) => poked && notify(`Poked “${node.title}”`));
	}
</script>

<div class="page">
	{#if !dashboard}
		<p class="muted">{loadFailed ? 'Could not load the dashboard. Try reloading.' : 'Loading…'}</p>
	{:else}
		<div class="tiles">
			{#each tiles as tile (tile.label)}
				<div class="tile">
					<div class="value {tile.tone}"><AnimatedNumber value={tile.value} /></div>
					<div class="label">{tile.label}</div>
				</div>
			{/each}
		</div>

		<div class="columns">
			<section aria-labelledby="active-heading">
				<div class="section-head">
					<span class="bar"></span>
					<h2 class="title" id="active-heading">Active</h2>
					<span class="meta">{actionableCount} actionable · {blockedActiveCount} blocked</span>
				</div>

				{#if dashboard.active.length}
					{#each activeTiers as group (group.tier)}
						<div class="tier-head">
							<span class="tier-bar" style:background={TIER_COLOR[group.tier]}></span>
							{group.tier}
							<span class="tier-count">{group.nodes.length}</span>
						</div>
						<div class="active-nodes">
							{#each group.nodes as node (node.id)}
								<NodeCard {node} feature />
							{/each}
						</div>
					{/each}
				{:else}
					<div class="empty">
						Nothing is active right now. Set a node's status to <strong>Active</strong> from its
						detail panel, or browse everything on the <a href="/board/organized">board</a>.
					</div>
				{/if}
			</section>

			<aside class="rail">
				<div class="panel">
					<h2 class="panel-title">Haven't touched in a while</h2>
					<p class="panel-sub">Active or queued, no poke for 14+ days. Poke to keep it.</p>
					{#if dashboard.stale.length}
						<ul class="rows">
							{#each dashboard.stale as node (node.id)}
								<li class="row">
									<span class="dot" style:background={accentColor(node)}></span>
									<button type="button" class="row-main" onclick={() => openNode(node.id)}>
										<span class="row-title">{node.title}</span>
										<span class="row-meta">
											{node.last_poked_at
												? `touched ${relativeDays(node.last_poked_at)}`
												: `never touched · ${shortDate(node.created_at)}`}
										</span>
									</button>
									{#if canPoke(node)}
										<Button variant="poke" onclick={() => poke(node)}>Poke</Button>
									{/if}
								</li>
							{/each}
						</ul>
					{:else}
						<p class="nothing">Nothing going stale.</p>
					{/if}
				</div>

				<div class="panel">
					<div class="panel-head">
						<h2 class="panel-title">Ideas</h2>
						<a class="more" href="/nodes">{dashboard.counts.ideas} ideas →</a>
					</div>
					<form class="capture" onsubmit={captureIdea}>
						<input
							class="field"
							placeholder="Capture an idea…"
							aria-label="New idea title"
							bind:value={ideaTitle}
						/>
						<Button variant="primary" type="submit" disabled={!ideaTitle.trim()}>Add</Button>
					</form>
					{#if dashboard.recent_ideas.length}
						<ul class="rows">
							{#each dashboard.recent_ideas as node (node.id)}
								<li class="row">
									<span class="dot" style:background={accentColor(node)}></span>
									<button type="button" class="row-main" onclick={() => openNode(node.id)}>
										<span class="row-title">{node.title}</span>
										<span class="row-meta">captured {shortDate(node.created_at)}</span>
									</button>
								</li>
							{/each}
						</ul>
					{/if}
				</div>

				{#if dashboard.paths.length}
					<div class="panel">
						<h2 class="panel-title">Paths</h2>
						<ul class="rows">
							{#each topPaths as node (node.id)}
								{@render pathItem(node)}
							{/each}
						</ul>
					</div>
				{/if}
			</aside>
		</div>
	{/if}
</div>

{#snippet pathItem(node: NodeResponse)}
	{@const progress = progressPair(node)}
	{@const inside = insideOf(node.id)}
	{@const collapsed = isPathCollapsed('dashboard', node.id)}
	<li>
		<div class="path-row">
			{#if inside.length}
				<CollapseToggle {collapsed} ontoggle={() => togglePathCollapsed('dashboard', node.id)} />
			{/if}
			<button type="button" class="path" onclick={() => openNode(node.id)}>
				<span class="row-title">{node.title}</span>
				{#if progress}
					<span class="path-progress">
						<ProgressBar value={progress[0]} total={progress[1]} height={8} />
						<span class="row-meta">{progressText(node)}</span>
					</span>
				{:else}
					<span class="row-meta">no steps yet</span>
				{/if}
			</button>
		</div>
		{#if !collapsed && inside.length}
			<ul class="rows nested">
				{#each inside as child (child.id)}
					{#if child.kind === 'path'}
						{@render pathItem(child)}
					{:else}
						<li>
							<button type="button" class="path-child" onclick={() => openNode(child.id)}>
								<span class="dot" style:background={accentColor(child)}></span>
								<span class="row-title">{child.title}</span>
								<span class="row-meta">{progressText(child) ?? child.status}</span>
							</button>
						</li>
					{/if}
				{/each}
			</ul>
		{/if}
	</li>
{/snippet}

<style>
	.page {
		padding: var(--page-pad);
		display: flex;
		flex-direction: column;
		gap: 18px;
	}

	.tiles {
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
	}

	.tile {
		flex: 1;
		min-width: 110px;
		background: var(--surface);
		border: var(--border-width) solid var(--line);
		padding: 13px 15px;
	}

	.value {
		font: 700 22px/1 var(--font-display);
	}

	.value.warn {
		color: var(--warn);
	}

	.value.ok {
		color: var(--ok);
	}

	.tile .label {
		margin-top: 6px;
	}

	.columns {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 380px;
		gap: 18px;
		align-items: start;
	}

	h2 {
		margin: 0;
	}

	.tier-head {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 18px;
		font: 700 10.5px/1 var(--font-mono);
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.tier-bar {
		width: 14px;
		height: 3px;
	}

	.tier-count {
		color: var(--ink-3);
	}

	.active-nodes {
		margin-top: 10px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.empty {
		margin-top: 12px;
		border: var(--border-width-hair) dashed var(--line);
		background: var(--surface);
		padding: 18px;
		color: var(--ink-3);
		font: 500 13px/1.55 var(--font-display);
	}

	.rail {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.rail .panel {
		padding: 18px 20px;
	}

	.panel-head {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.panel-title {
		font: 700 15px/1.1 var(--font-display);
	}

	.panel-sub {
		margin: 6px 0 0;
		font: 500 12.5px/1.45 var(--font-display);
		color: var(--ink-2);
	}

	.more {
		margin-left: auto;
		font: 700 13px/1 var(--font-display);
	}

	.capture {
		margin-top: 14px;
		display: flex;
		gap: 6px;
	}

	.rows {
		list-style: none;
		margin: 16px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.dot {
		width: 9px;
		height: 9px;
		flex: none;
	}

	.row-main,
	.path {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
	}

	.row-title {
		font: 700 14.5px/1.25 var(--font-display);
		color: var(--ink);
		overflow-wrap: anywhere;
	}

	.row-main:hover .row-title,
	.path:hover .row-title {
		color: var(--accent);
	}

	.row-meta {
		font: 500 12px/1.3 var(--font-mono);
		color: var(--ink-2);
	}

	.path {
		width: 100%;
		gap: 9px;
	}

	.path-row {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}

	.nested {
		margin: 10px 0 0 14px;
		padding-left: 12px;
		border-left: var(--border-width-hair) solid var(--line);
		gap: 10px;
	}

	.path-child {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
	}

	.path-child .row-title {
		flex: 1;
		min-width: 0;
		font-size: 13px;
	}

	.path-child:hover .row-title {
		color: var(--accent);
	}

	.path-progress {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.nothing {
		margin: 14px 0 0;
		font: 500 13px/1.4 var(--font-display);
		color: var(--ink-2);
	}

	@media (max-width: 900px) {
		.columns {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
