import { ApiError } from '$lib/api/client';

export type Toast = { id: number; message: string; tone: 'info' | 'error' };

export const toasts = $state<Toast[]>([]);

let nextToastId = 0;

export function notify(message: string, tone: Toast['tone'] = 'info') {
	const id = nextToastId++;
	toasts.push({ id, message, tone });
	setTimeout(() => dismiss(id), tone === 'error' ? 6000 : 3500);
}

export function dismiss(id: number) {
	const index = toasts.findIndex((toast) => toast.id === id);
	if (index !== -1) toasts.splice(index, 1);
}

export function notifyError(error: unknown) {
	notify(error instanceof ApiError ? error.message : 'Something went wrong', 'error');
}
