// raw.ts — graphlaw/raw: faithful passthrough of the JSON ABI.
// Never throws for ABI outcomes; the envelope comes back verbatim.
import { initGraphlaw } from './init.js';
import type { AbiRefused, AbiResult } from './types.js';

/** Sends one raw request object; returns the raw envelope. */
export async function callRaw(request: Record<string, unknown>): Promise<AbiResult<Record<string, unknown>>> {
  const wire = await initGraphlaw();
  return (await wire.call(request)) as AbiResult<Record<string, unknown>>;
}

/** Same as {@link callRaw} but for pre-serialized JSON strings. */
export async function callRawJson(requestJson: string): Promise<string> {
  const wire = await initGraphlaw();
  const response = await wire.callJson(requestJson);
  return response;
}

export type { AbiRefused, AbiResult };
