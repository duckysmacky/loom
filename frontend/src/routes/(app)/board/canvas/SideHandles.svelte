<script lang="ts">
	import { Handle, Position } from '@xyflow/svelte';
</script>

<!-- One handle per side, id = side. The canvas connects in loose mode, so
every handle both starts and ends connections: a drag runs from the item it
starts on. Graph edges always draw right -> left (related bottom -> top);
annotation arrows keep the sides they were drawn between. -->
<Handle type="source" position={Position.Top} id="top" />
<Handle type="source" position={Position.Right} id="right" />
<Handle type="source" position={Position.Bottom} id="bottom" />
<Handle type="source" position={Position.Left} id="left" />
