import { ApiError } from '$lib/api/client';
import { m } from '$lib/paraglide/messages';
import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeKind } from '$lib/types/NodeKind';
import type { NodeStatus } from '$lib/types/NodeStatus';

/**
 * Text for a failed API call. The server's messages are English; the ones a
 * user commonly hits are translated by status here. ponytail: other 4xx stay
 * English - add a stable `code` to the backend's `{"error": ...}` body if more
 * need translating.
 */
export function errorMessage(caught: unknown): string {
	if (!(caught instanceof ApiError)) return m.error_unreachable();
	if (caught.status === 429) return m.error_too_many_requests();
	if (caught.message === 'invalid email or password') return m.error_invalid_credentials();
	if (caught.message === 'email already registered') return m.error_email_taken();
	return caught.message;
}

/**
 * A message split around a value inside it, so the value can be wrapped in
 * markup: `{before}<strong>{value}</strong>{after}`.
 */
export function around(message: string, value: string): [string, string] {
	const index = message.indexOf(value);
	return index < 0 ? [message, ''] : [message.slice(0, index), message.slice(index + value.length)];
}

const KIND_LABEL: Record<NodeKind, () => string> = {
	idea: m.kind_idea,
	project: m.kind_project,
	study: m.kind_study,
	path: m.kind_path
};

const STATUS_LABEL: Record<NodeStatus, () => string> = {
	idea: m.status_idea,
	queued: m.status_queued,
	active: m.status_active,
	paused: m.status_paused,
	done: m.status_done
};

const FOCUS_LABEL: Record<NodeFocus, () => string> = {
	primary: m.focus_primary,
	secondary: m.focus_secondary,
	background: m.focus_background
};

export const kindLabel = (kind: NodeKind) => KIND_LABEL[kind]();
export const statusLabel = (status: NodeStatus) => STATUS_LABEL[status]();
export const focusLabel = (focus: NodeFocus) => FOCUS_LABEL[focus]();
