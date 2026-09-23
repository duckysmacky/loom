<script lang="ts">
	import NodeCard from '$lib/components/NodeCard.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { dashboardApi, nodesApi } from '$lib/api/endpoints';
	import { accentColor, relativeDays, shortDate } from '$lib/graph/display';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import { openNode } from '$lib/navigation';
	import type { DashboardResponse } from '$lib/types/DashboardResponse';

	let dashboard = $state<DashboardResponse | null>(null);
	let ideaTitle = $state('');

	// Refetch whenever the graph cache reloads, i.e. after any mutation.
	$effect(() => {
		void graph.version;
		dashboardApi
			.get()
			.then((response) => (dashboard = response))
			.catch(notifyError);
	});

	const actionableCount = $derived(
		dashboard?.primary.filter((node) => node.status === 'active' && !node.blocked).length ?? 0
	);
	const blockedPrimaryCount = $derived(
		dashboard?.primary.filter((node) => node.blocked).length ?? 0
	);

	const tiles = $derived(
		dashboard
			? [
					{ label: 'Total nodes', value: dashboard.counts.total, tone: '' },
					{ label: 'Active', value: dashboard.counts.by_status.active, tone: '' },
					{ label: 'Blocked', value: dashboard.counts.blocked, tone: 'warn' },
					{ label: 'Backlog', value: dashboard.counts.backlog, tone: '' },
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

	function poke(nodeId: string, title: string) {
		graph.mutate(() => nodesApi.poke(nodeId)).then((poked) => poked && notify(`Poked “${title}”`));
	}
</script>

<div class="page">
	{#if !dashboard}
		<p class="muted">Loading…</p>
	{:else}
		<div class="tiles">
			{#each tiles as tile (tile.label)}
				<div class="tile">
					<div class="value {tile.tone}">{tile.value}</div>
					<div class="label">{tile.label}</div>
				</div>
			{/each}
		</div>

		<div class="columns">
			<section aria-labelledby="primary-heading">
				<div class="section-head">
					<span class="bar"></span>
					<h2 class="title" id="primary-heading">Primary focus</h2>
					<span class="meta">{actionableCount} actionable · {blockedPrimaryCount} blocked</span>
				</div>

				{#if dashboard.primary.length}
					<div class="primary">
						{#each dashboard.primary as node (node.id)}
							<NodeCard {node} feature />
						{/each}
					</div>
				{:else}
					<div class="empty">
						Nothing is in the primary tier. Set a node's focus tier to <strong>Primary</strong> from
						its detail panel, or browse everything on the <a href="/board/organized">board</a>.
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
									<Button variant="poke" onclick={() => poke(node.id, node.title)}>Poke</Button>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="nothing">Nothing going stale.</p>
					{/if}
				</div>

				<div class="panel">
					<div class="panel-head">
						<h2 class="panel-title">Backlog</h2>
						<a class="more" href="/nodes">{dashboard.counts.backlog} ideas →</a>
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
					{#if dashboard.recent_backlog.length}
						<ul class="rows">
							{#each dashboard.recent_backlog as node (node.id)}
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

				{#if dashboard.containers.length}
					<div class="panel">
						<h2 class="panel-title">Paths</h2>
						<ul class="rows">
							{#each dashboard.containers as node (node.id)}
								{@const progress = node.container_progress!}
								<li>
									<button type="button" class="path" onclick={() => openNode(node.id)}>
										<span class="row-title">{node.title}</span>
										<span class="path-progress">
											<ProgressBar value={progress.done} total={progress.total} />
											<span class="row-meta">{progress.done}/{progress.total}</span>
										</span>
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/if}
			</aside>
		</div>
	{/if}
</div>

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
		grid-template-columns: minmax(0, 1fr) 300px;
		gap: 18px;
		align-items: start;
	}

	h2 {
		margin: 0;
	}

	.primary {
		margin-top: 12px;
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
		gap: 14px;
	}

	.panel-head {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.panel-title {
		font: 700 13px/1 var(--font-display);
	}

	.panel-sub {
		margin: 5px 0 0;
		font: 500 11.5px/1.4 var(--font-display);
		color: var(--ink-2);
	}

	.more {
		margin-left: auto;
		font: 700 12px/1 var(--font-display);
	}

	.capture {
		margin-top: 12px;
		display: flex;
		gap: 6px;
	}

	.rows {
		list-style: none;
		margin: 13px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 11px;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.dot {
		width: 7px;
		height: 7px;
		flex: none;
	}

	.row-main,
	.path {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
		border: none;
		background: none;
		padding: 0;
		text-align: left;
	}

	.row-title {
		font: 700 13px/1.2 var(--font-display);
		color: var(--ink);
		overflow-wrap: anywhere;
	}

	.row-main:hover .row-title,
	.path:hover .row-title {
		color: var(--accent);
	}

	.row-meta {
		font: 400 11px/1.3 var(--font-mono);
		color: var(--ink-2);
	}

	.path {
		width: 100%;
		gap: 7px;
	}

	.path-progress {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.nothing {
		margin: 12px 0 0;
		font: 500 12px/1.4 var(--font-display);
		color: var(--ink-2);
	}

	@media (max-width: 900px) {
		.columns {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
