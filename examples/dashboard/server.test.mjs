import assert from 'node:assert/strict';
import { test } from 'node:test';
import { decodeFrame } from './server.mjs';

test('decodes the attach frame protocol', () => {
  const frame = (channel, text) => new Uint8Array([channel, ...new TextEncoder().encode(text)]).buffer;
  assert.deepEqual(decodeFrame(frame(1, 'hi\n')), { kind: 'stdout', text: 'hi\n' });
  assert.deepEqual(decodeFrame(frame(2, 'oops')), { kind: 'stderr', text: 'oops' });
  assert.deepEqual(decodeFrame('{"type":"exit","exit_code":3}'), { kind: 'exit', code: 3 });
  assert.deepEqual(decodeFrame('{"type":"error","message":"no"}'), { kind: 'stderr', text: 'no\n' });
  assert.equal(decodeFrame(new ArrayBuffer(0)), null);
});
