<script lang="ts">
	import { goto } from '$app/navigation';
	import { cubicOut } from 'svelte/easing';
	import { fade, fly } from 'svelte/transition';
	import { ms } from '$lib/motion';
	import { page } from '$app/state';
	import AccentPicker from '$lib/components/ui/AccentPicker.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import PromoteDialog from '$lib/components/PromoteDialog.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import {
		accentColor,
		fromDateInput,
		incrementedProgress,
		kindChangeLosses,
		relativeDays,
		shortDate,
		toDateInput
	} from '$lib/graph/display';
	import { closeNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { UpdateNodeRequest } from '$lib/types/UpdateNodeRequest';
	import DetailChecklist from './DetailChecklist.svelte';
	import DetailConnections from './DetailConnections.svelte';
	import DetailContains from './DetailContains.svelte';
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
	let progressUnit = $state('');

	// Reset local drafts whenever a different node (or a fresh copy of it
	// after a mutation) is shown.
	$effect(() => {
		if (!node) return;
		titleDraft = node.title;
		progressCurrent = node.progress_current;
		progressTotal = node.progress_total;
		progressUnit = node.progress_unit ?? '';
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
		update(current, {
			progress_current: progressCurrent,
			progress_total: progressTotal,
			progress_unit: progressUnit.trim() || null
		});
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

	// Kinds are strict: only studies track progress, only projects have a checklist.
	const showProgress = $derived(node?.kind === 'study');

	// A kind change that would drop data waits for confirmation first.
	let pendingKind = $state<NodeResponse['kind'] | null>(null);
	const pendingLosses = $derived(node && pendingKind ? kindChangeLosses(node, pendingKind) : []);

	function requestKindChange(current: NodeResponse, kind: NodeResponse['kind']) {
		if (kindChangeLosses(current, kind).length) pendingKind = kind;
		else update(current, { kind });
	}

	function confirmKindChange(current: NodeResponse) {
		if (pendingKind) update(current, { kind: pendingKind });
		pendingKind = null;
	}
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
	<div class="scrim" onclick={closeNode} transition:fade={{ duration: ms(160) }}></div>
	<div
		class="detail"
		role="dialog"
		aria-modal="true"
		aria-label="Node detail"
		in:fly={{ y: 14, duration: ms(220), easing: cubicOut }}
		out:fade={{ duration: ms(120) }}
	>
		{#if !node}
			<div class="missing">
				<p>{graph.loaded ? 'This node no longer exists.' : 'Loading…'}</p>
				<Button variant="quiet" onclick={closeNode}>Close</Button>
			</div>
		{:else}
			{@const blocked = node.blocked && node.status !== 'done'}
			<div class="stripe" style:background={blocked ? 'var(--warn)' : accentColor(node)}></div>

			<header>
				<div class="meta-row">
					<span class="label">{node.kind}</span>
					<Badge status={node.status} blocked={node.blocked} />
					{#if blocked}<span class="computed">blocked is computed · not editable</span>{/if}
					<button type="button" class="close" aria-label="Close" onclick={closeNode}>✕</button>
				</div>
				<input
					class="title"
					aria-label="Title"
					bind:value={titleDraft}
					onblur={() => saveTitle(node)}
					onkeydown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
				/>
			</header>

			<div class="columns">
				<div class="main">
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
								<span class="of">/</span>
								<input
									class="field"
									type="number"
									min="1"
									aria-label="Total"
									bind:value={progressTotal}
								/>
								<input
									class="field unit"
									maxlength="40"
									placeholder="videos, chapters…"
									aria-label="What is being counted"
									bind:value={progressUnit}
								/>
								<Button type="submit" variant="poke">Save</Button>
								{#if node.progress_total !== null}
									<Button
										variant="poke"
										disabled={incrementedProgress(node) === null}
										onclick={() => update(node, { progress_current: incrementedProgress(node) })}
										>+1</Button
									>
									<Button
										variant="quiet"
										onclick={() =>
											update(node, {
												progress_current: null,
												progress_total: null,
												progress_unit: null
											})}>Clear</Button
									>
								{/if}
							</form>
						</section>
					{/if}

					<DetailNotes {node} />
					{#if node.kind === 'path'}
						<DetailContains {node} />
					{/if}
					{#if node.kind === 'project'}
						<DetailChecklist {node} />
					{/if}
					<DetailConnections {node} />
				</div>

				<aside class="side">
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
						<!-- The key re-renders the select, so a cancelled change snaps back. -->
						{#key pendingKind}
							<select
								class="field"
								value={pendingKind ?? node.kind}
								onchange={(event) =>
									requestKindChange(node, event.currentTarget.value as NodeResponse['kind'])}
							>
								<option value="idea">Idea</option>
								<option value="project">Project</option>
								<option value="study">Study</option>
								<option value="path">Path</option>
							</select>
						{/key}
					</label>
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
						<AccentPicker value={node.color} onchange={(color) => update(node, { color })} />
					</div>

					<DetailTopics {node} />

					<dl class="stats">
						<dt class="label">Last touched</dt>
						<dd>{node.last_poked_at ? relativeDays(node.last_poked_at) : 'never'}</dd>
						<dt class="label">Pokes</dt>
						<dd>{pokeCount ?? '–'}</dd>
						<dt class="label">Created</dt>
						<dd>{shortDate(node.created_at)}</dd>
					</dl>

					<!-- Stamped automatically on status changes; editable when the
					     real dates differ (e.g. something started before it was logged). -->
					<div class="dates">
						<label class="control">
							<span class="label">Started</span>
							<input
								class="field"
								type="date"
								value={toDateInput(node.started_at)}
								onchange={(event) =>
									update(node, { started_at: fromDateInput(event.currentTarget.value) })}
							/>
						</label>
						<label class="control">
							<span class="label">Completed</span>
							<input
								class="field"
								type="date"
								value={toDateInput(node.completed_at)}
								disabled={node.status !== 'done'}
								title={node.status === 'done'
									? undefined
									: 'Only done nodes have a completion date'}
								onchange={(event) =>
									update(node, { completed_at: fromDateInput(event.currentTarget.value) })}
							/>
						</label>
					</div>

					<div class="actions">
						<span class="label">Actions</span>
						{#if blocked}
							<span class="not-actionable">Not actionable yet</span>
						{:else if node.status === 'idea'}
							<Button variant="poke" onclick={() => (promoting = node)}>Move to board</Button>
						{:else if ['active', 'queued', 'paused'].includes(node.status)}
							<Button variant="poke" onclick={() => poke(node)}>Poke</Button>
						{/if}
						<Button
							variant="accent"
							onclick={() => goto(`/board/canvas?focus=${node.id}&node=${node.id}`)}
						>
							Open in canvas
						</Button>
						{#if node.status === 'archived'}
							<Button variant="quiet" onclick={() => update(node, { status: 'queued' })}>
								Restore
							</Button>
						{:else}
							<Button variant="quiet" onclick={() => update(node, { status: 'archived' })}>
								Archive
							</Button>
						{/if}
						{#if confirmingDelete}
							<Button variant="primary" onclick={() => remove(node)}>Delete for good</Button>
							<Button variant="quiet" onclick={() => (confirmingDelete = false)}>Keep</Button>
						{:else}
							<Button variant="quiet" onclick={() => (confirmingDelete = true)}>Delete</Button>
						{/if}
					</div>
				</aside>
			</div>
		{/if}
	</div>
{/if}

<PromoteDialog node={promoting} onclose={() => (promoting = null)} />

<Modal
	open={pendingKind !== null}
	onclose={() => (pendingKind = null)}
	label="Confirm kind change"
	width={440}
>
	<div class="confirm">
		<div class="label">Change kind to {pendingKind}?</div>
		<p>Each kind keeps different data. Switching will remove:</p>
		<ul>
			{#each pendingLosses as loss (loss)}
				<li>{loss}</li>
			{/each}
		</ul>
		<div class="confirm-actions">
			<Button variant="quiet" onclick={() => (pendingKind = null)}>Cancel</Button>
			<Button variant="primary" onclick={() => node && confirmKindChange(node)}>Change kind</Button>
		</div>
	</div>
</Modal>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 40;
		background: var(--scrim);
	}

	/* Centered card editor (Trello-style), not a side panel: the eye stays
	   in the middle of the screen. */
	.detail {
		position: fixed;
		z-index: 41;
		top: 5vh;
		left: 50%;
		transform: translateX(-50%);
		width: min(920px, calc(100vw - 32px));
		max-height: 90vh;
		overflow-y: auto;
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
		color: var(--ink);
	}

	.stripe {
		height: 5px;
	}

	.missing {
		padding: 24px;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 10px;
		color: var(--ink-2);
	}

	header {
		padding: 20px 28px 18px;
		border-bottom: var(--border-width-hair) solid var(--line);
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.meta-row {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.computed {
		font: 600 11px/1 var(--font-display);
		color: var(--ink-2);
	}

	.close {
		margin-left: auto;
		border: none;
		background: none;
		font: 600 17px/1 var(--font-display);
		color: var(--ink-2);
		padding: 2px;
	}

	.title {
		border: var(--border-width-hair) solid transparent;
		background: none;
		padding: 3px 5px;
		margin: 0 -6px;
		font: 700 26px/1.2 var(--font-display);
		color: var(--ink);
	}

	.title:hover,
	.title:focus {
		border-color: var(--line);
		outline: none;
	}

	.columns {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 270px;
	}

	.main {
		padding: 22px 28px 28px;
		display: flex;
		flex-direction: column;
		gap: 26px;
		font-size: 14px;
	}

	.side {
		padding: 22px 22px 28px;
		border-left: var(--border-width-hair) solid var(--line);
		background: var(--surface-2);
		display: flex;
		flex-direction: column;
		gap: 18px;
	}

	.control {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.control .field {
		padding: 8px 28px 8px 10px;
		font-size: 13.5px;
		background-color: var(--surface);
	}

	.control :global(.segmented) {
		display: flex;
	}

	.control :global(.segmented button) {
		flex: 1;
		padding: 8px 4px;
	}

	.progress {
		margin-top: 10px;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.progress .field {
		width: 84px;
		padding: 8px 10px;
	}

	.progress .field.unit {
		width: auto;
		flex: 1;
		min-width: 120px;
	}

	.of {
		font: 600 13px/1 var(--font-display);
		color: var(--ink-2);
	}

	.dates {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.dates .field {
		padding: 7px 10px;
		font-size: 13px;
		min-width: 0;
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

	.stats {
		margin: 0;
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 9px 12px;
		align-items: baseline;
	}

	.stats dd {
		margin: 0;
		text-align: right;
		font: 700 13.5px/1 var(--font-display);
	}

	.actions {
		display: flex;
		flex-direction: column;
		gap: 7px;
	}

	.actions :global(.button) {
		width: 100%;
		justify-content: flex-start;
	}

	.not-actionable {
		font: 700 12.5px/1 var(--font-display);
		color: var(--ink-2);
		border: var(--border-width-hair) solid var(--line);
		padding: 10px 12px;
	}

	@media (max-width: 760px) {
		.detail {
			top: 0;
			max-height: 100vh;
			width: 100vw;
		}

		.columns {
			grid-template-columns: minmax(0, 1fr);
		}

		.side {
			border-left: none;
			border-top: var(--border-width-hair) solid var(--line);
		}
	}
</style>
