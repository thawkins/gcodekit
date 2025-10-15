- Implement editor core file src/gcodeedit/editor.rs (buffer, cursor, selection, undo/redo, folding).  
     - Incremental tokenizer + parser service (debounced background task) with unit tests and DOCBLOCKs.  
     - Auto-completion API and UI (G/M codes, params, context-aware suggestions).  
     - Code folding, virtualized line rendering, and performance tuning for large files.  
     - Rule configuration UI (enable/disable, set severity, select GRBL version) and persistence.  
     - Editor <-> Visualizer integration (line mapping, back-plot stepping, highlight sync).  
     - Find/replace UI refinements, keyboard mappings, and accessibility polish.  
     - Tests: tokenizer, rules, completions, editor UI behavior, and documentation for new modules.

