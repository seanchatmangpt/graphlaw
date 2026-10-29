All sources read. Here is the spec.

---

# SPEC — GL28-1: OCEL 2.0 event envelope emitted by `lib/beam4pm_graphlaw_bridge.ex`

## 0. Source of truth (verified line numbers)

| Concern | File | Lines |
|---|---|---|
| Referential-integrity validation | `/Users/sac/beam4pm/lib/beam4pm_ocel.ex` | `validate_envelope/2` :99-128 |
| Wire decode (single record) | `/Users/sac/beam4pm/lib/beam4pm_ocel_ingest.ex` | `decode_one/2` :128-153, `decode_relationships/1` :99-126, `present/3` :84-89 |
| Struct required fields | `/Users/sac/beam4pm/lib/beam4pm_types.ex` | `OcelEvent.new/1` :11845-11859, `OcelObject.new/1` :11873-11885, `OcelRelationship.new/1` :11925-11937, `OcelAttribute` :11807-11831 |
| General envelope decode gap | `/Users/sac/beam4pm/lib/beam4pm_codec.ex` | `from_map(:ocel_event,...)` :11185-11196, `from_known_fields/3` :14773-14783 |
| Telemetry contract | `/Users/sac/graphlaw/graphlaw_ex/lib/graphlaw_ex/telemetry.ex` | `admit/3` :55-91, moduledoc :1-32 |

## 1. Envelope shape the bridge must emit

Two decode paths consume bridge output — the envelope must satisfy BOTH:

1. **`BeamPM.Ocel.decode/1`** (`beam4pm_ocel.ex:165-176`) — requires top-level `"events"` and `"objects"` lists (`%{"events" => ..., "objects" => ...}`); extra top-level keys (`"eventTypes"`, `"objectTypes"`, produced by `BeamPM.Ocel.encode/1` :144-149) are ignored by the pattern match and are safe to include.
2. **`POST /ocel/events` / `POST /ocel/objects`** (`beam4pm_ocel_ingest.ex:31-77`) — accepts `{"events": [...]}` (or a single event map) and `{"objects": [...]}` (or single object map).

Per-record wire keys (from `decode_one/2`):
- **Event**: `event_id`, `event_type`, `event_time`, `attributes` (optional), `relationships` (optional list).
- **Object**: `object_id`, `object_type`, `attributes` (optional), `relationships` (optional list).
- **Relationship**: `qualifier`, `object_id` — nested under its owning event (`"relationships": [...]`), never a top-level collection (:91-98).

## 2. Field-by-field mapping table (telemetry → OCEL event field → validation rule)

`[:graphlaw_ex, :admit, :stop]` payload (telemetry.ex:66-70): measurements `%{duration: integer}` (native monotonic units); metadata `%{op: String.t(), server: term, outcome: :ok | :refused, refusal_code: atom | nil}`. Note `:stop` never carries `:exception` — that is a separate event (:exception, metadata adds `kind`/`reason`/`stacktrace`, `outcome: :exception`, `refusal_code: nil`).

| Telemetry key | OCEL event field | Validation rule (enforced where) |
|---|---|---|
| `metadata.op` (String) | `event_type` | Key present + value non-null (`OcelEvent.new/1` :11848-11850 via ingest `present/3` :84-89 drops JSON `null`). Violation → `{:error, {:missing_field, :event_type}}` |
| `metadata.op` + `:start` `measurements.system_time` (or handler-minted UUID) | `event_id` | Present + non-null string; no uniqueness check in code — uniqueness is a bridge convention only |
| Handler-minted wall clock (ISO 8601 string), or `:start` `system_time` converted via `System.convert_time_unit(t, :native, :millisecond)` then `DateTime.from_unix!/2` | `event_time` | Present + non-null string. No format check — `:passthrough` kind (`beam4pm_codec.ex:11191`), so any string decodes; ISO 8601 is a bridge convention |
| (no telemetry source — mint in handler) | `attributes` | Optional; `map()` passthrough, zero inner validation (integers/atoms-as-strings allowed) |
| `measurements.duration` (integer, native units) | `attributes["duration_native"]` | Passthrough; store integer as-is, do not convert |
| `metadata.outcome` (`:ok`/`:refused`) | `attributes["outcome"]` | `Atom.to_string/1` → `"ok"`/`"refused"`; inner-map values are unvalidated passthrough |
| `metadata.refusal_code` (atom, `nil` when outcome `:ok` or non-`Refusal` error) | `attributes["refusal_code"]` | Omit key when `nil`; else `Atom.to_string/1` |
| `metadata.server` (term) | object in envelope `objects[]` + `relationships[].object_id` | Object: `object_id`, `object_type` present + non-null (`OcelObject.new/1` :11876-11877). Relationship: `object_id` must resolve to a same-envelope object or `validate_envelope/2` returns `{:error, {:dangling_relationships, ...}}` |
| `[:graphlaw_ex, :engine, :admit]` `wasm_sha256` (from `EngineLoad` receipt) | `attributes["wasm_sha256"]` on the engine object | Passthrough |
| (bridge-minted role name, e.g. `"performed_by"`) | `relationships[].qualifier` | Present + non-null (`OcelRelationship.new/1` :11928-11929); missing → `{:error, {:relationship_missing_field, field}}`; `relationships` present but not a list → `{:error, {:invalid_field, :relationships}}` (:124) |

## 3. Exact minimal valid envelope JSON (passes `validate_envelope/2`)

Minimal admissible form (attributes omitted is legal — absence ≠ missing-field; only the keys above are required):

```json
{
  "events": [
    {
      "event_id": "admit_stop-convert_plan-1000",
      "event_type": "convert_plan",
      "event_time": "2026-10-28T00:00:00Z"
    }
  ],
  "objects": [
    { "object_id": "graphlaw_ex", "object_type": "graphlaw_engine" }
  ]
}
```

Zero relationships → `validate_envelope/2` returns `:ok` trivially (no rels to dangle). This shape is also accepted single-record by both ingest routes and by `BeamPM.Ocel.decode/1`.

Recommended bridge emission (same validation outcome, with relationship wiring — one E2O rel on the event, both rels resolving to the in-envelope object):

```json
{
  "events": [
    {
      "event_id": "admit_stop-convert_plan-1000",
      "event_type": "convert_plan",
      "event_time": "2026-10-28T00:00:00Z",
      "attributes": {
        "outcome": "ok",
        "refusal_code": null,
        "duration_native": 4211
      },
      "relationships": [
        { "qualifier": "performed_by", "object_id": "graphlaw_ex" }
      ]
    }
  ],
  "objects": [
    {
      "object_id": "graphlaw_ex",
      "object_type": "graphlaw_engine",
      "attributes": { "wasm_sha256": "<hex>" }
    }
  ]
}
```

Both are valid; the second additionally survives `validate_envelope/2` WITH referential integrity exercised (rel target `"graphlaw_ex"` is in the envelope's object set).

## 4. Gotchas the bridge must not trip

1. **JSON `null` on any required field = missing field.** `present/3` (`beam4pm_ocel_ingest.ex:84-89`) drops nil values before `new/1`, so `"event_time": null` yields `{:error, {:missing_field, :event_time}}` and a 422 — same as omitting the key. Never emit `null` for `event_id`/`event_type`/`event_time`/`object_id`/`object_type`/`qualifier`/relationship `object_id`.
2. **`BeamPM.Ocel.decode/1` silently discards `relationships`.** `from_map(:ocel_event, ...)` reads only the 4 scalar keys (`beam4pm_codec.ex:11185-11196`); the nested `"relationships"` key is not in the spec list, so a decode→`validate_envelope/2` round trip via that path sees zero rels and always returns `:ok`. The bridge must decode relationships itself (mirror `decode_relationships/1`, `beam4pm_ocel_ingest.ex:99-126`) before calling `validate_envelope/2`, or it has not actually validated referential integrity.
3. **`nil` inside `attributes` is fine** — only top-level record fields pass through `present/3`; the `attributes` map is unvalidated passthrough. `"refusal_code": null` inside `attributes` decodes cleanly (as in the recommended example).
4. **`:stop` outcome is only `:ok`/`:refused`** (`classify/1`, telemetry.ex:93-96) — exceptions surface on the separate `:exception` event, never at `:stop`. If the bridge wants an exception lane, it consumes `[:graphlaw_ex, :admit, :exception]` separately.
5. **No wall-clock timestamp exists at `:stop`.** `duration` is monotonic; the only wall-clock-ish value in the span is `:start`'s `measurements.system_time` (`System.system_time()`, telemetry.ex:60). Either correlate `:start`/`:stop` in the bridge handler or mint `event_time` at handler time.