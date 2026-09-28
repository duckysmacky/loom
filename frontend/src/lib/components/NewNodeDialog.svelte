<script lang="ts">
	import AccentPicker from '$lib/components/ui/AccentPicker.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Chip from '$lib/components/ui/Chip.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import TopicToggles from '$lib/components/TopicToggles.svelte';
	import { checklistApi, edgesApi, nodesApi } from '$lib/api/endpoints';
	import { accentColor, fromDateInput } from '$lib/graph/display';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import { overlays } from '$lib/stores/ui.svelte';
	import type { EdgeKind } from '$lib/types/EdgeKind';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { NodeStatus } from '$lib/types/NodeStatus';

	/**
	 * The full constructor, next to quick capture's title-and-go: every field
	 * a new node can start with, plus its first connections. Saved through the
	 * regular endpoints; if any step fails the half-built node is deleted
	 * again (all or nothing, like the MCP subgraph tool).
	 */

	// Outgoing only, read from the new node's side; incoming ones are added
	// later from the detail panel.
	const RELATIONS: { kind: EdgeKind; label: string }[] = [
		{ kind: 'requires', label: 'Requires' },
		{ kind: 'precedes', label: 'Comes before' },
		{ kind: 'part_of', label: 'Inside path' },
		{ kind: 'related', label: 'Related to' }
	];

	let kind = $state<NodeKind>('project');
	let title = $state('');
	let notes = $state('');
	let color = $state<string | null>(null);
	let status = $state<Extract<NodeStatus, 'idea' | 'queued' | 'active'>>('queued');
	let focus = $state<NodeFocus>('secondary');
	let started = $state('');
	let checklist = $state('');
	let progressCurrent = $state<number | null>(null);
	let progressTotal = $state<number | null>(null);
	let progressUnit = $state('');
	let links = $state<{ kind: EdgeKind; targetId: string }[]>([]);
	let linkKind = $state<EdgeKind>('requires');
	let linkTarget = $state('');
	let topicIds = $state<string[]>([]);
	let saving = $state(false);

	$effect(() => {
		if (!overlays.newOpen) return;
		kind = prefs.captureKind === 'idea' ? 'project' : prefs.captureKind;
		title = notes = started = checklist = progressUnit = linkTarget = '';
		color = progressCurrent = progressTotal = null;
		status = 'queued';
		focus = prefs.captureFocus;
		links = [];
		linkKind = 'requires';
		topicIds = [];
	});

	const committed = $derived(kind !== 'idea');
	const inPath = $derived(links.some((link) => link.kind === 'part_of'));
	const relations = $derived(
		RELATIONS.filter((relation) => relation.kind !== 'part_of' || !inPath)
	);
	const candidates = $derived(
		graph.nodes
			.filter((node) => node.status !== 'archived')
			// Only paths can contain nodes.
			.filter((node) => linkKind !== 'part_of' || node.kind === 'path')
			.filter((node) => !links.some((link) => link.kind === linkKind && link.targetId === node.id))
			.toSorted((left, right) => left.title.localeCompare(right.title))
	);
	const checklistItems = $derived(
		checklist
			.split('\n')
			.map((line) => line.trim())
			.filter(Boolean)
	);

	function addLink() {
		if (!linkTarget) return;
		links = [...links, { kind: linkKind, targetId: linkTarget }];
		linkTarget = '';
		if (linkKind === 'part_of') linkKind = 'requires';
	}

	async function create(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = title.trim();
		if (!trimmed) return;
		saving = true;
		const created = await graph.mutate(async () => {
			const node = await nodesApi.create({
				kind,
				title: trimmed,
				notes: notes.trim() || null,
				color,
				// Ideas take neither status nor focus; a path's status is derived.
				...(committed ? { focus } : {}),
				...(committed && kind !== 'path' ? { status } : {}),
				...(kind === 'study' && progressTotal
					? {
							progress_current: progressCurrent ?? 0,
							progress_total: progressTotal,
							progress_unit: progressUnit.trim() || null
						}
					: {})
			});
			try {
				if (kind === 'project') {
					for (const item of checklistItems) await checklistApi.add(node.id, item);
				}
				for (const link of links) {
					await edgesApi.create({
						from_node_id: node.id,
						to_node_id: link.targetId,
						kind: link.kind
					});
				}
				for (const topicId of topicIds) await nodesApi.attachTopic(node.id, topicId);
				if (committed && kind !== 'path' && started) {
					await nodesApi.update(node.id, { started_at: fromDateInput(started) });
				}
			} catch (error) {
				// Deleting the node cascades to whatever was already attached.
				await nodesApi.remove(node.id).catch(() => {});
				throw error;
			}
			return node;
		});
		saving = false;
		if (created) {
			overlays.newOpen = false;
			notify(`Created “${created.title}”`, 'info', {
				label: 'Open',
				run: () => openNode(created.id)
			});
		}
	}
</script>

<Modal
	open={overlays.newOpen}
	onclose={() => (overlays.newOpen = false)}
	label="New node"
	width={600}
>
	<form class="new" onsubmit={create}>
		<div class="label">New node</div>
		<!-- svelte-ignore a11y_autofocus -->
		<input class="title" placeholder="Title" aria-label="Title" autofocus bind:value={title} />

		<div class="row">
			<div class="group">
				<span class="label">Kind</span>
				<SegmentedControl
					label="Kind"
					value={kind}
					onchange={(value) => (kind = value)}
					options={[
						{ value: 'idea', label: 'Idea' },
						{ value: 'project', label: 'Project' },
						{ value: 'study', label: 'Study' },
						{ value: 'path', label: 'Path' }
					]}
				/>
			</div>
			<div class="group">
				<span class="label">Accent</span>
				<AccentPicker value={color} onchange={(value) => (color = value)} size="small" />
			</div>
		</div>

		{#if committed}
			<div class="row">
				{#if kind !== 'path'}
					<div class="group">
						<span class="label">Status</span>
						<SegmentedControl
							label="Status"
							value={status}
							onchange={(value) => (status = value)}
							options={[
								{ value: 'idea', label: 'Idea' },
								{ value: 'queued', label: 'Queued' },
								{ value: 'active', label: 'Active' }
							]}
						/>
					</div>
				{/if}
				<div class="group">
					<span class="label">Focus</span>
					<SegmentedControl
						label="Focus"
						value={focus}
						onchange={(value) => (focus = value)}
						options={[
							{ value: 'primary', label: 'Primary' },
							{ value: 'secondary', label: 'Secondary' },
							{ value: 'background', label: 'Background' }
						]}
					/>
				</div>
				{#if kind !== 'path'}
					<label class="group">
						<span class="label">Started</span>
						<input class="field" type="date" bind:value={started} />
					</label>
				{/if}
			</div>
		{/if}

		<label class="group wide">
			<span class="label">Notes</span>
			<textarea class="field" rows="3" placeholder="Markdown" bind:value={notes}></textarea>
		</label>

		{#if kind === 'project'}
			<label class="group wide">
				<span class="label">Checklist · one item per line</span>
				<textarea class="field" rows="3" bind:value={checklist}></textarea>
			</label>
		{:else if kind === 'study'}
			<div class="row">
				<label class="group">
					<span class="label">Done</span>
					<input class="field number" type="number" min="0" bind:value={progressCurrent} />
				</label>
				<label class="group">
					<span class="label">Total</span>
					<input class="field number" type="number" min="1" bind:value={progressTotal} />
				</label>
				<label class="group">
					<span class="label">Unit</span>
					<input class="field" placeholder="chapters" maxlength="40" bind:value={progressUnit} />
				</label>
			</div>
		{/if}

		<div class="group wide">
			<span class="label">Connections</span>
			{#if links.length}
				<div class="chips">
					{#each links as link, index (`${link.kind}-${link.targetId}`)}
						{@const target = graph.nodeById.get(link.targetId)}
						<Chip
							label="{RELATIONS.find((relation) => relation.kind === link.kind)
								?.label} · {target?.title}"
							color={target ? accentColor(target) : null}
							onremove={() => (links = links.filter((_, other) => other !== index))}
						/>
					{/each}
				</div>
			{/if}
			<div class="link-row">
				<select class="field" aria-label="Relation" bind:value={linkKind}>
					{#each relations as relation (relation.kind)}
						<option value={relation.kind}>{relation.label}</option>
					{/each}
				</select>
				<select class="field target" aria-label="Node" bind:value={linkTarget}>
					<option value="">Pick a node…</option>
					{#each candidates as candidate (candidate.id)}
						<option value={candidate.id}>{candidate.title}</option>
					{/each}
				</select>
				<Button variant="quiet" onclick={addLink} disabled={!linkTarget}>Add</Button>
			</div>
		</div>

		<TopicToggles bind:selected={topicIds} />

		<div class="actions">
			<span class="hint">esc to close</span>
			<Button type="submit" variant="primary" disabled={!title.trim() || saving}>Create</Button>
		</div>
	</form>
</Modal>

<style>
	.new {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-height: 85vh;
		overflow-y: auto;
	}

	.title {
		border: none;
		border-bottom: var(--border-width) solid var(--frame);
		background: none;
		padding: 6px 0 8px;
		font: 700 20px/1.2 var(--font-display);
		color: var(--ink);
	}

	.title:focus {
		outline: none;
		border-bottom-color: var(--accent);
	}

	.row {
		display: flex;
		gap: 18px;
		flex-wrap: wrap;
	}

	.group {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-start;
	}

	.wide {
		align-items: stretch;
	}

	textarea {
		resize: vertical;
	}

	.number {
		width: 90px;
	}

	.chips {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.link-row {
		display: flex;
		gap: 8px;
	}

	.target {
		flex: 1;
		min-width: 0;
	}

	.actions {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 10px;
		padding-top: 14px;
		border-top: var(--border-width-hair) solid var(--line);
	}

	.hint {
		margin-right: auto;
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}
</style>
