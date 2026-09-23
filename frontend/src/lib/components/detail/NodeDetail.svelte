<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import PromoteDialog from '$lib/components/PromoteDialog.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { ACCENT_PALETTE, accentColor, relativeDays, shortDate } from '$lib/graph/display';
	import { closeNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { UpdateNodeRequest } from '$lib/types/UpdateNodeRequest';
	import DetailConnections from './DetailConnections.svelte';
	import DetailNotes from './DetailNotes.svelte';
	import DetailTopics from './DetailTopics.svelte';

	/** Node detail slide-over, deep-linked via `?node=<id>` on any page. */
	const nodeId = $derived(page.url.searchParams.get('node'));
	const node = $derived(nodeId ? graph.nodeById.get(nodeId) : undefined);

	let pokeCount = $state<number | null>(null);
	let confirmingDelete = $state(false);
	let promoting = $state<NodeResponse | null>(null);
	let titleDraft = $state('');
	let progressCurrent = $state<number | null>(null);
	let progressTotal = $state<number | null>(null);

	// Reset local drafts whenever a different node (or a fresh copy of it
	// after a mutation) is shown.
	$effect(() => {
		if (!node) return;
		titleDraft = node.title;
		progressCurrent = node.progress_current;
		progressTotal = node.progress_total;
	});

	$effect(() => {
		confirmingDelete = false;
		if (!nodeId) return;
		void graph.version;
		nodesApi
			.pokes(nodeId)
			.then((pokes) => (pokeCount = pokes.length))
			.catch(() => (pokeCount = null));
	});

	const update = (current: NodeResponse, patch: UpdateNodeRequest) =>
		graph.mutate(() => nodesApi.update(current.id, patch));

	function saveTitle(current: NodeResponse) {
		const title = titleDraft.trim();
		if (!title) titleDraft = current.title;
		else if (title !== current.title) update(current, { title });
	}

	function saveProgress(current: NodeResponse) {
		if (progressCurrent === null || progressTotal === null) return;
		update(current, { progress_current: progressCurrent, progress_total: progressTotal });
	}

	async function poke(current: NodeResponse) {
		if (await graph.mutate(() => nodesApi.poke(current.id))) notify(`Poked “${current.title}”`);
	}

	async function remove(current: NodeResponse) {
		try {
			await nodesApi.remove(current.id);
			closeNode();
			await graph.load();
			notify(`Deleted “${current.title}”`);
		} catch (error) {
			notifyError(error);
		}
	}

	const showProgress = $derived(
		node ? node.kind === 'course' || node.progress_total !== null : false
	);
</script>

<svelte:window
	onkeydown={(event) => {
		// Esc closes the panel unless a dialog or text field is handling it.
		if (event.key !== 'Escape' || !nodeId || document.querySelector('dialog[open]')) return;
		const target = event.target as HTMLElement;
		if (target.matches('input, textarea, select')) return;
		closeNode();
	}}
/>

{#if nodeId}
	<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
	<div class="scrim" onclick={closeNode}></div>
	<aside class="detail" aria-label="Node detail">
		{#if !node}
			<div class="missing">
				<p>{graph.loaded ? 'This node no longer exists.' : 'Loading…'}</p>
				<Button variant="quiet" onclick={closeNode}>Close</Button>
			</div>
		{:else}
			{@const kind = node.kind}
			{@const blocked = node.blocked && node.status !== 'done'}
			<div class="stripe" style:background={blocked ? 'var(--warn)' : accentColor(node)}></div>

			<header>
				<div class="meta-row">
					<span class="label">{kind}</span>
					<Badge status={node.status} blocked={node.blocked} />
					{#if blocked}<span class="computed">computed · not editable</span>{/if}
					<button type="button" class="close" aria-label="Close" onclick={closeNode}>✕</button>
				</div>

				<input
					class="title"
					aria-label="Title"
					bind:value={titleDraft}
					onblur={() => saveTitle(node)}
					onkeydown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
				/>

				<div class="controls">
					<label class="control">
						<span class="label">Status</span>
						<select
							class="field"
							value={node.status}
							onchange={(event) =>
								update(node, { status: event.currentTarget.value as NodeResponse['status'] })}
						>
							{#each ['idea', 'queued', 'active', 'paused', 'done', 'archived'] as status (status)}
								<option value={status}>{status[0].toUpperCase() + status.slice(1)}</option>
							{/each}
						</select>
					</label>
					<label class="control">
						<span class="label">Kind</span>
						<select
							class="field"
							value={node.kind}
							onchange={(event) =>
								update(node, { kind: event.currentTarget.value as NodeResponse['kind'] })}
						>
							<option value="idea">Idea</option>
							<option value="project">Project</option>
							<option value="course">Course</option>
							<option value="path">Path</option>
						</select>
					</label>
				</div>

				<div class="controls">
					<div class="control">
						<span class="label">Focus tier</span>
						<SegmentedControl
							label="Focus tier"
							value={node.focus}
							onchange={(focus) => update(node, { focus })}
							options={[
								{ value: 'primary', label: 'Primary' },
								{ value: 'secondary', label: 'Secondary' },
								{ value: 'background', label: 'Background' }
							]}
						/>
					</div>
					<div class="control">
						<span class="label">Accent</span>
						<div class="swatches">
							{#each ACCENT_PALETTE as color (color)}
								<button
									type="button"
									class="swatch"
									class:chosen={node.color === color}
									style:background={color}
									aria-label="Accent {color}"
									aria-pressed={node.color === color}
									onclick={() => update(node, { color: node.color === color ? null : color })}
								></button>
							{/each}
						</div>
					</div>
				</div>
			</header>

			<div class="body">
				{#if showProgress}
					<section>
						<div class="label">Progress</div>
						<form
							class="progress"
							onsubmit={(event) => {
								event.preventDefault();
								saveProgress(node);
							}}
						>
							<input
								class="field"
								type="number"
								min="0"
								aria-label="Done so far"
								bind:value={progressCurrent}
							/>
							<span class="of">of</span>
							<input
								class="field"
								type="number"
								min="1"
								aria-label="Total"
								bind:value={progressTotal}
							/>
							<Button type="submit" variant="poke">Save</Button>
							{#if node.progress_total !== null}
								<Button
									variant="quiet"
									onclick={() => update(node, { progress_current: null, progress_total: null })}
									>Clear</Button
								>
							{/if}
						</form>
					</section>
				{/if}

				<DetailNotes {node} />
				<DetailConnections {node} />
				<DetailTopics {node} />

				<div class="stats">
					<div class="stat">
						<div class="label">Last touched</div>
						<div class="value">
							{node.last_poked_at ? relativeDays(node.last_poked_at) : 'never'}
						</div>
					</div>
					<div class="stat">
						<div class="label">Pokes</div>
						<div class="value">{pokeCount ?? '–'}</div>
					</div>
					<div class="stat">
						<div class="label">Created</div>
						<div class="value">{shortDate(node.created_at)}</div>
					</div>
					<div class="stat">
						<div class="label">Started</div>
						<div class="value">{node.started_at ? shortDate(node.started_at) : '–'}</div>
					</div>
					<div class="stat">
						<div class="label">Completed</div>
						<div class="value">{node.completed_at ? shortDate(node.completed_at) : '–'}</div>
					</div>
				</div>
			</div>

			<footer>
				{#if blocked}
					<span class="not-actionable">Not actionable yet</span>
				{:else if node.kind === 'idea' && node.status === 'idea'}
					<Button variant="poke" onclick={() => (promoting = node)}>Promote</Button>
				{:else if ['active', 'queued', 'paused'].includes(node.status)}
					<Button variant="poke" onclick={() => poke(node)}>Poke</Button>
				{/if}
				<span class="spacer"></span>
				{#if confirmingDelete}
					<Button variant="quiet" onclick={() => (confirmingDelete = false)}>Keep</Button>
					<Button variant="primary" onclick={() => remove(node)}>Delete for good</Button>
				{:else}
					<Button variant="quiet" onclick={() => (confirmingDelete = true)}>Delete</Button>
					{#if node.status === 'archived'}
						<Button variant="quiet" onclick={() => update(node, { status: 'queued' })}>
							Restore
						</Button>
					{:else}
						<Button variant="quiet" onclick={() => update(node, { status: 'archived' })}>
							Archive
						</Button>
					{/if}
					<Button
						variant="accent"
						onclick={() => goto(`/board/canvas?focus=${node.id}&node=${node.id}`)}
					>
						Open in canvas
					</Button>
				{/if}
			</footer>
		{/if}
	</aside>
{/if}

<PromoteDialog node={promoting} onclose={() => (promoting = null)} />

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 40;
		background: var(--scrim);
	}

	.detail {
		position: fixed;
		top: 0;
		right: 0;
		bottom: 0;
		z-index: 41;
		width: min(490px, 100vw);
		background: var(--surface);
		border-left: var(--border-width) solid var(--line);
		display: flex;
		flex-direction: column;
		color: var(--ink);
	}

	.stripe {
		height: 4px;
		flex: none;
	}

	.missing {
		padding: 22px;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 10px;
		color: var(--ink-2);
	}

	header {
		padding: 18px 22px 16px;
		border-bottom: var(--border-width-hair) solid var(--line);
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.meta-row {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.computed {
		font: 600 10.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	.close {
		margin-left: auto;
		border: none;
		background: none;
		font: 600 15px/1 var(--font-display);
		color: var(--ink-2);
		padding: 2px;
	}

	.title {
		border: var(--border-width-hair) solid transparent;
		background: none;
		padding: 2px 4px;
		margin: -3px -5px 0;
		font: 700 22px/1.2 var(--font-display);
		color: var(--ink);
	}

	.title:hover,
	.title:focus {
		border-color: var(--line);
		outline: none;
	}

	.controls {
		display: flex;
		align-items: flex-start;
		gap: 18px;
		flex-wrap: wrap;
	}

	.control {
		display: flex;
		flex-direction: column;
		gap: 7px;
	}

	.control .field {
		padding: 6px 28px 6px 9px;
		font-size: 12.5px;
		width: auto;
	}

	.swatches {
		display: flex;
		gap: 5px;
		flex-wrap: wrap;
		max-width: 230px;
	}

	.swatch {
		width: 18px;
		height: 18px;
		border: var(--border-width) solid transparent;
		padding: 0;
	}

	.swatch.chosen {
		border-color: var(--ink);
		outline: 1px solid var(--surface);
		outline-offset: -3px;
	}

	.body {
		flex: 1;
		overflow-y: auto;
		padding: 18px 22px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.progress {
		margin-top: 10px;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.progress .field {
		width: 76px;
		padding: 6px 8px;
	}

	.of {
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.stats {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
		gap: 8px;
	}

	.stat {
		border: var(--border-width-hair) solid var(--line);
		padding: 11px 13px;
	}

	.value {
		margin-top: 7px;
		font: 700 14px/1 var(--font-display);
	}

	footer {
		border-top: var(--border-width-hair) solid var(--line);
		padding: 14px 22px;
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.spacer {
		flex: 1;
	}

	.not-actionable {
		font: 700 12px/1 var(--font-display);
		color: var(--ink-2);
		border: var(--border-width-hair) solid var(--line);
		padding: 9px 12px;
	}
</style>
