<script lang="ts" module>
	import { BookOpen, Hammer, Lightbulb, Route } from '@lucide/svelte';
	import type { NodeKind } from '$lib/types/NodeKind';

	export const KIND_ICON = { project: Hammer, study: BookOpen, idea: Lightbulb, path: Route };
</script>

<script lang="ts">
	import Icon from '$lib/components/ui/Icon.svelte';
	import { KIND_GLYPH } from '$lib/graph/display';

	/** A node kind's mark: its icon, or its glyph in the glyph icon style. Takes the surrounding color. */
	let { kind, size = 13 }: { kind: NodeKind; size?: number } = $props();
</script>

<Icon icon={KIND_ICON[kind]} glyph={KIND_GLYPH[kind]} {size} />
