import { describe, expect, it, vi } from 'vitest';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/') } }));

const { nextPath } = await import('./navigation');

describe('nextPath', () => {
	it('returns a same-origin path', () => {
		expect(nextPath('?next=%2Foauth%2Fauthorize%3Fclient_id%3Dabc')).toBe(
			'/oauth/authorize?client_id=abc'
		);
	});

	it('falls back without a next param', () => {
		expect(nextPath('')).toBe('/dashboard');
	});

	it('refuses off-site targets', () => {
		for (const next of [
			'https://evil.example',
			'//evil.example',
			'/\\evil.example',
			'javascript:alert(1)'
		]) {
			expect(nextPath(`?next=${encodeURIComponent(next)}`)).toBe('/dashboard');
		}
	});
});
