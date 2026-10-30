<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Zap } from '@lucide/svelte';
	import { KIND_ICON } from '$lib/components/KindMark.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import TopicToggles from '$lib/components/TopicToggles.svelte';
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
	let status = $state<Extract<NodeStatus, 'idea' | 'queued' | 'active'>>('queued');
	let topicIds = $state<string[]>([]);
	let saving = $state(false);

	// Fresh form with the configured defaults (Settings > Features)
	// every time it opens. Ideas carry no status/focus; everything else
	// defaults to queued.
	$effect(() => {
		if (!overlays.captureOpen) return;
		title = '';
		kind = prefs.captureKind;
		focus = prefs.captureFocus;
		status = 'queued';
		topicIds = [];
	});

	async function capture(event: SubmitEvent) {
		event.preventDefault();
		const trimmed = title.trim();
		if (!trimmed) return;
		saving = true;
		const created = await graph.mutate(async () => {
			const node = await nodesApi.create(
				// Ideas take neither status nor focus; a path's status is derived.
				kind === 'idea'
					? { kind, title: trimmed }
					: { kind, title: trimmed, focus, ...(kind === 'path' ? {} : { status }) }
			);
			for (const topicId of topicIds) await nodesApi.attachTopic(node.id, topicId);
			return node;
		});
		saving = false;
		if (created) {
			overlays.captureOpen = false;
			notify(m.capture_done({ title: created.title }), 'info', {
				label: m.action_open(),
				run: () => openNode(created.id)
			});
		}
	}
</script>

<Modal
	open={overlays.captureOpen}
	onclose={() => (overlays.captureOpen = false)}
	label={m.quick_capture()}
	width={500}
>
	<form class="capture" onsubmit={capture}>
		<div class="label">{m.quick_capture()}</div>
		<!-- svelte-ignore a11y_autofocus -->
		<input
			class="title"
			placeholder={m.capture_placeholder()}
			aria-label={m.field_title()}
			autofocus
			bind:value={title}
		/>

		<div class="row">
			<div class="group">
				<span class="label">{m.field_kind()}</span>
				<SegmentedControl
					label={m.field_kind()}
					value={kind}
					onchange={(value) => (kind = value)}
					options={[
						{ value: 'idea', label: m.kind_idea(), icon: KIND_ICON.idea },
						{ value: 'project', label: m.kind_project(), icon: KIND_ICON.project },
						{ value: 'study', label: m.kind_study(), icon: KIND_ICON.study },
						{ value: 'path', label: m.kind_path(), icon: KIND_ICON.path }
					]}
				/>
			</div>
			{#if kind !== 'idea'}
				<div class="group">
					<span class="label">{m.field_focus()}</span>
					<SegmentedControl
						label={m.field_focus()}
						value={focus}
						onchange={(value) => (focus = value)}
						options={[
							{ value: 'primary', label: m.focus_primary() },
							{ value: 'secondary', label: m.focus_secondary() },
							{ value: 'background', label: m.focus_background() }
						]}
					/>
				</div>
				{#if kind !== 'path'}
					<div class="group">
						<span class="label">{m.field_status()}</span>
						<SegmentedControl
							label={m.field_status()}
							value={status}
							onchange={(value) => (status = value)}
							options={[
								{ value: 'idea', label: m.status_idea() },
								{ value: 'queued', label: m.status_queued() },
								{ value: 'active', label: m.status_active() }
							]}
						/>
					</div>
				{/if}
			{/if}
		</div>

		<TopicToggles bind:selected={topicIds} />

		<div class="actions">
			<span class="hint">{m.capture_hint()}</span>
			<Button type="submit" variant="primary" disabled={!title.trim() || saving} icon={Zap}
				>{m.capture_submit()}</Button
			>
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
