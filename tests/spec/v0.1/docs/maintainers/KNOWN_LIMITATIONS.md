# Known limitations

- Synthetic fixtures prove mechanical consistency only; they do not prove product value.
- There are no real user experiments in v0.1.
- There is no live GitHub or RSS adapter in this repository.
- Exploration has no sufficiently clear deterministic selection semantics yet, so `explorationBudget` and `exploration.used` are excluded from v0.1.
- The ranking formula is a replaceable baseline: weighted finite signals summed by Lens weights.
- Possible failure conditions include explanations that users do not trust, analyzers producing weak evidence, and Lens weights that do not transfer to real tasks.
- A future iteration should stop or revise this direction if conformance passes but real users cannot predict, audit, or improve their feeds.
