## Problem Description
STEP 2A spec gap: POST_GATE runs `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'`, which fails with `ModuleNotFoundError: No module named 'yaml'` because PyYAML is not installed in the container's `python3`. Location: `STEPPED_IMPLEMENTATION_PLAN.md` (STEP 2A POST_GATE).
Lapse: L4 other
Kind: scope-lapse
Found during: STEP 2A (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
