# Governance

UEAS follows a formal governance model designed to ensure the long-term
stability and community ownership of the standard.

## Current Model: BDFL

During the bootstrap phase (Epochs 1-3), the project founder holds final
decision authority as Benevolent Dictator For Life (BDFL).

## Future Model: Technical Steering Committee (TSC)

After Epoch 3 completion, authority transitions to a Technical Steering
Committee.

**TSC Composition:** 5 members, elected annually by contributors with at
least one ratified RFC.

**Voting:**
- Specification changes: Consensus (no unresolved objections) via RFC process
- Maintainer addition: Majority vote
- Release go/no-go: Majority vote + CI gates green
- Code of Conduct enforcement: 2/3 supermajority

## RFC Process

All specification changes proceed through the formal RFC process documented
in [docs/rfcs/README.md](docs/rfcs/README.md). The state machine is:

**Draft → Review → Ratification → Implementation**

## Maintainers

Maintainers are contributors who have demonstrated sustained, high-quality
contributions. See [CONTRIBUTING.md Section 12](docs/CONTRIBUTING.md#12-maintainer-guide)
for the complete maintainer lifecycle.
