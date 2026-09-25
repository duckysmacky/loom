<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import { overlays } from '$lib/stores/ui.svelte';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { NodeStatus } from '$lib/types/NodeStatus';

	let title = $state('');
	let kind = $state<NodeKind>('idea');
	let focus = $state<NodeFocus>('secondary');
	// Only these three make sense straight out of quick capture; anything
	// else (paused/done/archived) is a status change made later.
	let status = $state<Extract<NodeStatus, 'idea' | 'queued' | 'active'>>('idea');
	let topicIds = $state<string[]>([]);
	let saving = $state(false);

	// Fresh form with the configured defaults (Settings > Functionality)
	// every time it opens. Backlog = status idea, for any kind - an idea
	// defaults there, everything else defaults to queued.
	$effect(() => {
		if (!overlays.captureOpen) return;
		title = '';
		kind = prefs.captureKind;
		focus = prefs.captureFocus;
		status = kind === 'idea' ? 'idea' : 'queued';
		topicIds = [];
	});

	function toggleTopic(topicId: string) {
		topicIds = topicIds.includes(topicId)
			? topicIds.filter((id) => id !== topicId)
			: [...topicIds, topicId];
	}

	async function capture(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = title.trim();
		if (!trimmed) return;
		saving = true;
		const created = await graph.mutate(async () => {
			const node = await nodesApi.create({ kind, title: trimmed, focus, status });
			for (const topicId of topicIds) await nodesApi.attachTopic(node.id, topicId);
			return node;
		});
		saving = false;
		if (created) {
			overlays.captureOpen = false;
			notify(`Captured “${created.title}”`, 'info', {
				label: 'Open',
				run: () => openNode(created.id)
			});
		}
	}
</script>

<Modal
	open={overlays.captureOpen}
	onclose={() => (overlays.captureOpen = false)}
	label="Quick capture"
	width={500}
>
	<form class="capture" onsubmit={capture}>
		<div class="label">Quick capture</div>
		<!-- svelte-ignore a11y_autofocus -->
		<input
			class="title"
			placeholder="What's on your mind?"
			aria-label="Title"
			autofocus
			bind:value={title}
		/>

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
			<div class="group">
				<span class="label">Status</span>
				<SegmentedControl
					label="Status"
					value={status}
					onchange={(value) => (status = value)}
					options={[
						{ value: 'idea', label: 'Backlog' },
						{ value: 'queued', label: 'Queued' },
						{ value: 'active', label: 'Active' }
					]}
				/>
			</div>
		</div>

		{#if graph.topics.length}
			<div class="group">
				<span class="label">Topics</span>
				<div class="topics">
					{#each graph.topics as topic (topic.id)}
						<button
							type="button"
							class="topic"
							class:selected={topicIds.includes(topic.id)}
							aria-pressed={topicIds.includes(topic.id)}
							onclick={() => toggleTopic(topic.id)}
						>
							{#if topic.color}<span class="swatch" style:background={topic.color}></span>{/if}
							{topic.name}
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<div class="actions">
			<span class="hint">↵ to capture · esc to close</span>
			<Button type="submit" variant="primary" disabled={!title.trim() || saving}>Capture</Button>
		</div>
	</form>
</Modal>

<style>
	.capture {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 16px;
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

	.topics {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.topic {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 6px 9px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.topic.selected {
		border-color: var(--ink);
		background: var(--ink);
		color: var(--on-ink);
	}

	.swatch {
		width: 7px;
		height: 7px;
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
