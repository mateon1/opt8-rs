# opt8

This project is intended as an excercise and experiment in optimizing compilers and JITs.

## ROADMAP

- [ ] Create proper documentation
- [ ] Naïve interpreter
- [ ] Symbolic interpreter
- [ ] Extract basic blocks
- [ ] Extract CFGs
- [ ] Optimizations:
* * [ ] Document invariants and necessary checks for practical optimization (self-modifying code, etc.)
* * [ ] Constant propagation
* * [ ] Dead store elimination
* * * [ ] On registers
* * * [ ] On memory
* * [ ] Dead code elimination
* * [ ] Peephole optimizations
* * * [ ] SMT queries to verify optimization correctness
* * * [ ] Synthesis-based superoptimization

