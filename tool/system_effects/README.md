# System Effects Tooling

Generators for the system-wide warfare buff feature (environmental effects:
wormholes, metaliminal/volatile storms, and the native dbuff environments).

Pipeline (run from the crate root, requires `uv` and Python 3.12+):

```sh
# 1. Extract effect-beacon data from our own per-server FSD dumps
#    (data/resources/<server>/fsd). Prints a per-server value report;
#    cross-server divergence is a warning, not an error.
#    Writes tool/system_effects/out/beacons.json.
uv run --with msgpack python tool/system_effects/extract_wormhole.py

# 2. Generate the engine data patch data/patches/system_effects.yaml and
#    the catalog resolution rules tool/system_effects/out/environment_catalog.json.
uv run --with pyyaml --with msgpack python tool/system_effects/gen_system_effects.py

# 3. Generate the app catalog. The output path is passed explicitly; from the
#    monorepo workspace root it is
#    apps/eve-fit-assistant/lib/pages/fit/components/system_effect/system_effect_catalog.dart
#    (../../apps/... from the crate root).
python3 tool/system_effects/gen_dart_catalog.py \
  --output ../../apps/eve-fit-assistant/lib/pages/fit/components/system_effect/system_effect_catalog.dart

# 4. Rebuild the engine data (see ../../README.md).
uv run -m data.convert
```

`extract_wormhole.py` reads each server's `typedogma.msgpack` (plus the
workspace en-US localization pickle for skill/group names; run
`./x build data <server>` first). All three servers currently carry identical
beacon dogma; the app nevertheless resolves preset values from the ACTIVE
snapshot at runtime, so future per-server divergence is handled by
construction.

The catalog carries no buff values:

- Wormhole and storm presets reference their effect beacon plus a
  buff-ID -> beacon-attribute mapping.
- Abyssal weather and hazard cloud presets expand the beacon's
  `warfareBuff1-4ID/Value` attribute pairs directly.
- Sovereignty/Triglavian/insurgency presets stay hardcoded (accepted risk):
  their beacons carry no usable attributes on any server (pyfa hardcodes
  them too).

The app-side expansion logic lives in the hand-written
`system_effect_resolver.dart` next to the generated catalog.
