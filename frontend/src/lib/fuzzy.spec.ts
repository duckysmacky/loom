import { describe, expect, it } from 'vitest';
import { fuzzyScore } from './fuzzy';

describe('fuzzyScore', () => {
	it('matches everything on an empty query', () => {
		expect(fuzzyScore('  ', 'anything')).toBe(0);
	});

	it('ranks substrings above subsequences, earlier above later', () => {
		const substring = fuzzyScore('open', 'Custom OpenGL renderer')!;
		const early = fuzzyScore('open', 'OpenGL (Cherno)')!;
		const subsequence = fuzzyScore('ogl', 'OpenGL (Cherno)')!;
		expect(early).toBeGreaterThan(substring);
		expect(substring).toBeGreaterThan(subsequence);
	});

	it('rejects out-of-order or missing characters', () => {
		expect(fuzzyScore('lgo', 'OpenGL')).toBeNull();
		expect(fuzzyScore('xyz', 'OpenGL')).toBeNull();
	});
});
