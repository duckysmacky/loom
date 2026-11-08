<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { boardApi } from '$lib/api/endpoints';
	import { accentColor, shortDate } from '$lib/graph/display';
	import {
		DAY_MS,
		DEFAULT_PX_PER_DAY,
		barMarks,
		clampZoom,
		nodeBarSpans,
		pokeOffsets,
		rangeWidth,
		scrollAfterZoom,
		timelineEligible,
		timelineRange,
		weekStarts,
		xOf
	} from '$lib/graph/timeline';
	import { focusLabel, kindLabel, statusLabel } from '$lib/i18n';
	import { openNode } from '$lib/navigation';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { ActivePeriodResponse } from '$lib/types/ActivePeriodResponse';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { Span } from '$lib/graph/timeline';
	import type { PokeResponse } from '$lib/types/PokeResponse';

	/** Width of the node-name column, which stays fixed while the time axis scrolls. */
	const NAME_COLUMN = 240;
	const ZOOM_STEP = 1.25;
	/** Narrower bars show no label. */
	const MIN_LABEL_WIDTH = 44;

	// Settings > Appearance > Timeline: fill per tier, border colours, paused rows.
	const styles = $derived(prefs.timelineStyles);
	const visible = $derived(
		graph.nodes.filter(
			(node) => matchesBoardFilters(node) && timelineEligible(node, styles.show_paused)
		)
	);
	const started = $derived(
		visible
			.filter((node) => node.started_at)
			.toSorted((left, right) => left.started_at!.localeCompare(right.started_at!))
	);
	const notStarted = $derived(visible.filter((node) => !node.started_at));
	const range = $derived(timelineRange(started));
	const now = Date.now();

	// Every poke and active period the caller owns, refetched whenever the
	// graph reloads after a mutation (same trigger `DetailChecklist.svelte`
	// uses) - grouped by node for O(1) lookup per row.
	let pokesByNode = $state(new Map<string, PokeResponse[]>());
	let periodsByNode = $state(new Map<string, ActivePeriodResponse[]>());
	$effect(() => {
		void graph.version;
		boardApi
			.timeline()
			.then((timeline) => {
				// Plain Maps, built once then assigned wholesale to the $state
				// variables above - not mutated in place, so reactivity doesn't
				// need SvelteMap here.
				// eslint-disable-next-line svelte/prefer-svelte-reactivity
				const pokes = new Map<string, PokeResponse[]>();
				for (const poke of timeline.pokes) {
					pokes.set(poke.node_id, [...(pokes.get(poke.node_id) ?? []), poke]);
				}
				// eslint-disable-next-line svelte/prefer-svelte-reactivity
				const periods = new Map<string, ActivePeriodResponse[]>();
				for (const period of timeline.periods) {
					periods.set(period.node_id, [...(periods.get(period.node_id) ?? []), period]);
				}
				pokesByNode = pokes;
				periodsByNode = periods;
			})
			.catch(notifyError);
	});

	let pxPerDay = $state(DEFAULT_PX_PER_DAY);
	let viewport: HTMLDivElement | undefined = $state();

	const axisWidth = $derived(rangeWidth(range, pxPerDay));
	const weeks = $derived(weekStarts(range));
	// Thin week labels out when zoomed far out so they don't collide.
	const labelEvery = $derived(pxPerDay * 7 < 34 ? 4 : pxPerDay * 7 < 64 ? 2 : 1);
	const showDays = $derived(pxPerDay >= 26);
	const todayX = $derived(xOf(now, range, pxPerDay));

	/** Zoom to `next` px/day, keeping the date under `pointerX` (axis-relative) in place. */
	function zoomTo(next: number, pointerX = (viewport?.clientWidth ?? NAME_COLUMN) / 2) {
		if (!viewport) return;
		const clamped = clampZoom(next);
		const axisPointer = Math.max(0, pointerX - NAME_COLUMN);
		const scrollLeft = scrollAfterZoom(viewport.scrollLeft, axisPointer, pxPerDay, clamped);
		pxPerDay = clamped;
		// Wait for the wider/narrower axis to render before scrolling.
		requestAnimationFrame(() => viewport && (viewport.scrollLeft = scrollLeft));
	}

	function scrollToToday(behavior: ScrollBehavior = 'smooth') {
		if (!viewport) return;
		const axisVisible = viewport.clientWidth - NAME_COLUMN;
		viewport.scrollTo({ left: todayX - axisVisible * 0.7, behavior });
	}

	function fitAll() {
		if (!viewport) return;
		const days = (range.end - range.start) / DAY_MS;
		pxPerDay = clampZoom((viewport.clientWidth - NAME_COLUMN) / days);
		requestAnimationFrame(() => viewport?.scrollTo({ left: 0, behavior: 'smooth' }));
	}

	// Land on today the first time there's something to show.
	let scrolledInitially = false;
	$effect(() => {
		if (!viewport || !graph.loaded || scrolledInitially) return;
		scrolledInitially = true;
		requestAnimationFrame(() => scrollToToday('instant'));
	});

	// Ctrl+wheel zooms around the pointer. Registered by hand: it must be
	// non-passive to stop the browser's own page zoom.
	$effect(() => {
		if (!viewport) return;
		const element = viewport;
		const onWheel = (event: WheelEvent) => {
			if (!event.ctrlKey) return;
			event.preventDefault();
			const factor = event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
			zoomTo(pxPerDay * factor, event.clientX - element.getBoundingClientRect().left);
		};
		element.addEventListener('wheel', onWheel, { passive: false });
		return () => element.removeEventListener('wheel', onWheel);
	});

	// Drag anywhere on the chart to pan. A drag that actually moved swallows
	// the click that follows, so releasing over a bar doesn't open it.
	let pan: { x: number; y: number; left: number; top: number; moved: boolean } | null = null;
	let suppressClick = false;
	let panning = $state(false);

	function onPointerDown(event: PointerEvent) {
		if (!viewport || event.button !== 0) return;
		if ((event.target as HTMLElement).closest('.name, .toolbar')) return;
		pan = {
			x: event.clientX,
			y: event.clientY,
			left: viewport.scrollLeft,
			top: viewport.scrollTop,
			moved: false
		};
	}

	function onPointerMove(event: PointerEvent) {
		if (!pan || !viewport) return;
		const dx = event.clientX - pan.x;
		const dy = event.clientY - pan.y;
		if (!pan.moved && Math.hypot(dx, dy) < 4) return;
		if (!pan.moved) viewport.setPointerCapture(event.pointerId);
		pan.moved = true;
		panning = true;
		viewport.scrollLeft = pan.left - dx;
		viewport.scrollTop = pan.top - dy;
	}

	function onPointerUp() {
		suppressClick = pan?.moved ?? false;
		pan = null;
		panning = false;
	}

	// A bar is filled by the focus the node had back then (a paused node was
	// active); done / blocked describe where it is now, so only its latest bar
	// carries them, as borders.
	const LEGEND_TIERS: [NodeFocus, () => string][] = [
		['primary', m.legend_primary],
		['secondary', m.legend_secondary],
		['background', m.legend_background]
	];
	const tierOf = (focus: NodeFocus | null): NodeFocus => focus ?? 'background';
	const timelineVars = $derived(
		[
			`--tl-primary:${styles.primary.color ?? 'var(--ink)'}`,
			`--tl-secondary:${styles.secondary.color ?? 'var(--ink-2)'}`,
			`--tl-background:${styles.background.color ?? 'var(--ink-2)'}`,
			`--tl-done:${styles.done_color ?? 'var(--ok)'}`,
			`--tl-blocked:${styles.blocked_color ?? 'var(--hazard)'}`
		].join(';')
	);

	function barCaption(node: NodeResponse): string {
		const from = shortDate(node.started_at!);
		return node.completed_at
			? m.timeline_range({ from, to: shortDate(node.completed_at) })
			: m.timeline_since({ from });
	}

	function periodCaption(period: ActivePeriodResponse): string {
		const from = shortDate(period.started_at);
		return period.ended_at
			? m.timeline_range({ from, to: shortDate(period.ended_at) })
			: m.timeline_since({ from });
	}

	/** A bar's text: the focus it had; a ghost (idle since pausing) has none. */
	function barLabel(node: NodeResponse, span: Span): string {
		if (span.kind === 'ghost') return '';
		return span.focus ? focusLabel(span.focus) : kindLabel(node.kind);
	}

	function barTitle(node: NodeResponse, span: Span, caption: string): string {
		const status = node.status ? statusLabel(node.status) : kindLabel(node.kind);
		const focus = span.focus ? focusLabel(span.focus) : kindLabel(node.kind);
		return span.kind === 'ghost'
			? `${node.title} · ${status}`
			: `${node.title} · ${status} · ${focus} · ${caption}`;
	}

	const dayLabel = (time: number) => new Date(time).getDate();
</script>

<div class="page" style={timelineVars}>
	<div class="toolbar">
		<button type="button" aria-label={m.zoom_out()} onclick={() => zoomTo(pxPerDay / ZOOM_STEP)}
			>−</button
		>
		<span class="zoom">{m.timeline_zoom({ px: Math.round(pxPerDay * 7) })}</span>
		<button type="button" aria-label={m.zoom_in()} onclick={() => zoomTo(pxPerDay * ZOOM_STEP)}
			>+</button
		>
		<button type="button" onclick={() => scrollToToday()}>{m.timeline_today()}</button>
		<button type="button" onclick={fitAll}>{m.action_fit()}</button>
		<span class="hint">{m.timeline_hint()}</span>
	</div>

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="viewport"
		class:panning
		bind:this={viewport}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
		onclickcapture={(event) => {
			if (!suppressClick) return;
			suppressClick = false;
			event.stopPropagation();
			event.preventDefault();
		}}
		style:--px-per-day="{pxPerDay}px"
	>
		<div class="sheet" style:width="{NAME_COLUMN + axisWidth}px">
			<div class="row head">
				<div class="name corner label">{m.timeline_node()}</div>
				<div class="axis" style:width="{axisWidth}px">
					{#each weeks as week, index (week)}
						{#if index % labelEvery === 0}
							<span class="week" style:left="{xOf(week, range, pxPerDay)}px">
								{shortDate(new Date(week).toISOString())}
							</span>
						{/if}
						{#if showDays}
							{#each [1, 2, 3, 4, 5, 6] as offset (offset)}
								<span
									class="day"
									style:left="{xOf(week + offset * DAY_MS, range, pxPerDay)}px"
									style:width="{pxPerDay}px">{dayLabel(week + offset * DAY_MS)}</span
								>
							{/each}
						{/if}
					{/each}
				</div>
			</div>

			{#each started as node (node.id)}
				{@const periods = periodsByNode.get(node.id) ?? []}
				{@const spans = nodeBarSpans(node, periods, range, pxPerDay, now, styles.show_paused)}
				{@const pokeXs = pokeOffsets(
					(pokesByNode.get(node.id) ?? []).map((poke) => poke.poked_at),
					range,
					pxPerDay
				)}
				<div class="row">
					<button type="button" class="name" onclick={() => openNode(node.id)}>
						<span class="dot" style:background={accentColor(node)}></span>
						<span class="name-text">
							<span class="title">{node.title}</span>
							<span class="meta">{node.focus ?? node.kind} · {barCaption(node)}</span>
						</span>
					</button>
					<div class="track" class:days={showDays} style:width="{axisWidth}px">
						{#each spans as span, index (index)}
							{@const caption = periods[index] ? periodCaption(periods[index]) : barCaption(node)}
							{@const marks = barMarks(node, span, index === spans.length - 1)}
							{@const tier = tierOf(span.focus)}
							<button
								type="button"
								class="bar look tier-{tier} fill-{styles[tier].fill}"
								class:mark-done={marks.done}
								class:mark-blocked={marks.blocked}
								class:ghost={marks.ghost}
								style:left="{span.left}px"
								style:width="{span.width}px"
								title={barTitle(node, span, caption)}
								onclick={() => openNode(node.id)}
							>
								<!-- A one-day bar is too narrow for text. -->
								{#if span.width >= MIN_LABEL_WIDTH}<span>{barLabel(node, span)}</span>{/if}
							</button>
						{/each}
						{#each pokeXs as x, index (index)}
							<span class="poke" style:left="{x}px" title={m.timeline_poked()}></span>
						{/each}
					</div>
				</div>
			{:else}
				<div class="empty">
					{visible.length ? m.timeline_empty_filtered() : m.timeline_empty()}
				</div>
			{/each}

			<div class="today" style:left="{NAME_COLUMN + todayX}px" title={m.timeline_today()}></div>
		</div>
	</div>

	<div class="legend">
		{#each LEGEND_TIERS as [tier, label] (tier)}
			<span><span class="swatch look tier-{tier} fill-{styles[tier].fill}"></span>{label()}</span>
		{/each}
		<span><span class="swatch look tier-primary mark-done"></span>{m.legend_done()}</span>
		<span><span class="swatch look tier-primary mark-blocked"></span>{m.legend_blocked()}</span>
		{#if styles.show_paused}
			<span><span class="swatch look ghost"></span>{m.legend_paused()}</span>
		{/if}
		<span><span class="tick"></span>{m.legend_poke()}</span>
		<span><span class="today-mark"></span>{m.legend_today()}</span>
		<span class="note">{m.legend_history()}</span>
	</div>

	{#if notStarted.length}
		<section class="not-started">
			<div class="section-head">
				<span class="bar-mark"></span>
				<h2 class="title-head">{m.timeline_not_started()}</h2>
				<span class="meta-head">{m.timeline_no_start({ count: notStarted.length })}</span>
			</div>
			<div class="chips">
				{#each notStarted as node (node.id)}
					<button type="button" class="chip" onclick={() => openNode(node.id)}>
						<span class="dot" style:background={accentColor(node)}></span>
						{node.title}
						<span class="status">{node.status ?? node.kind}</span>
					</button>
				{/each}
			</div>
		</section>
	{/if}
</div>

<style>
	.page {
		padding: var(--page-pad);
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.toolbar button {
		min-width: 28px;
		height: 28px;
		padding: 0 10px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		color: var(--ink);
		font: 700 12px/1 var(--font-mono);
	}

	.toolbar button:hover {
		border-color: var(--ink);
	}

	.zoom {
		min-width: 110px;
		text-align: center;
		font: 600 11px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.hint {
		margin-left: 8px;
		font: 600 11.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	/* The scroll container: both axes scroll, header row and name column stick. */
	.viewport {
		position: relative;
		overflow: auto;
		max-height: calc(100vh - 290px);
		min-height: 260px;
		background: var(--surface);
		border: var(--border-width) solid var(--line);
		cursor: grab;
		overscroll-behavior: contain;
	}

	.viewport.panning {
		cursor: grabbing;
		user-select: none;
	}

	.sheet {
		position: relative;
		min-width: 100%;
	}

	.row {
		display: flex;
		border-bottom: var(--border-width-hair) solid var(--line);
	}

	.head {
		position: sticky;
		top: 0;
		z-index: 3;
		background: var(--surface-2);
	}

	.name {
		position: sticky;
		left: 0;
		z-index: 2;
		width: 240px;
		flex: none;
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 11px 16px;
		border: none;
		border-right: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		text-align: left;
		color: var(--ink);
		cursor: pointer;
	}

	.corner {
		z-index: 4;
		background: var(--surface-2);
		cursor: default;
	}

	.dot {
		width: 7px;
		height: 7px;
		flex: none;
	}

	.name-text {
		min-width: 0;
	}

	.title {
		display: block;
		font: 700 13px/1.2 var(--font-display);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.name:hover .title {
		color: var(--accent);
	}

	.meta {
		display: block;
		margin-top: 4px;
		font: 500 9.5px/1.2 var(--font-mono);
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.axis {
		position: relative;
		flex: none;
		height: 34px;
	}

	.week,
	.day {
		position: absolute;
		top: 0;
		padding: 7px 5px 0;
		border-left: var(--border-width-hair) solid var(--line);
		white-space: nowrap;
		font: 500 10px/1 var(--font-mono);
		color: var(--ink-2);
		height: 100%;
	}

	.week {
		color: var(--ink);
		font-weight: 600;
	}

	.day {
		padding-top: 20px;
		border-left-color: var(--surface-2);
		text-align: center;
		padding-left: 0;
		padding-right: 0;
	}

	/* Week grid (and day grid when zoomed in) drawn as a background, so it
	   costs nothing however long the axis gets. */
	.track {
		position: relative;
		flex: none;
		min-height: 48px;
		background-image: repeating-linear-gradient(
			90deg,
			var(--line) 0 1px,
			transparent 1px calc(7 * var(--px-per-day))
		);
	}

	.track.days {
		background-image:
			repeating-linear-gradient(
				90deg,
				var(--line) 0 1px,
				transparent 1px calc(7 * var(--px-per-day))
			),
			repeating-linear-gradient(90deg, var(--surface-2) 0 1px, transparent 1px var(--px-per-day));
	}

	.bar {
		position: absolute;
		top: 17px;
		height: 14px;
		border: none;
		padding: 0 6px;
		overflow: hidden;
		text-align: left;
		font: 700 9.5px/14px var(--font-mono);
		white-space: nowrap;
		cursor: pointer;
	}

	/* Shared by bars and legend swatches. The fill comes from the focus tier
	   (--bar, via .tier-*) and its fill style; done / blocked are borders on top. */
	.look {
		--bar: var(--tl-primary, var(--ink));
		background: var(--bar);
		color: var(--on-ink);
	}

	.tier-secondary {
		--bar: var(--tl-secondary, var(--ink-2));
	}

	.tier-background {
		--bar: var(--tl-background, var(--ink-2));
	}

	.fill-striped {
		background: repeating-linear-gradient(135deg, var(--bar) 0 6px, #fff 6px 12px);
		color: var(--ink);
	}

	.fill-outline {
		background: var(--surface);
		border: 1.5px dashed var(--bar);
		color: var(--bar);
		line-height: 11px;
	}

	.mark-done {
		outline: 2px solid var(--tl-done, var(--ok));
	}

	.mark-blocked {
		outline: 2px dashed var(--tl-blocked, var(--hazard));
	}

	/* Idle since pausing: the old "background" look. */
	.ghost {
		background: var(--surface);
		border: 1.5px dashed var(--ink-2);
		color: var(--ink-2);
	}

	.poke {
		position: absolute;
		top: 12px;
		width: 2px;
		height: 24px;
		background: var(--accent);
		pointer-events: none;
	}

	.today {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--warn);
		opacity: 0.55;
		z-index: 1;
		pointer-events: none;
	}

	.empty {
		position: sticky;
		left: 0;
		width: fit-content;
		padding: 28px 18px;
		color: var(--ink-2);
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 14px;
		flex-wrap: wrap;
		font: 600 11.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	.legend > span {
		display: flex;
		align-items: center;
		gap: 7px;
	}

	.swatch {
		width: 18px;
		height: 9px;
		display: inline-block;
	}

	.tick,
	.today-mark {
		width: 2px;
		height: 12px;
		background: var(--accent);
		display: inline-block;
	}

	.today-mark {
		background: var(--warn);
	}

	.note {
		margin-left: auto;
	}

	.not-started {
		margin-top: 8px;
	}

	.not-started .bar-mark {
		width: 4px;
		height: 15px;
		background: var(--line);
	}

	.section-head {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.title-head {
		margin: 0;
		font: 700 13px/1 var(--font-display);
		letter-spacing: 0.07em;
		text-transform: uppercase;
	}

	.meta-head {
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.chips {
		margin-top: 12px;
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 11px;
		background: var(--surface);
		border: var(--border-width-hair) dashed var(--line);
		font: 700 12.5px/1 var(--font-display);
		color: var(--ink);
	}

	.chip:hover {
		border-color: var(--ink);
	}

	.status {
		font: 500 9.5px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	@media (max-width: 700px) {
		.hint {
			display: none;
		}
	}
</style>
