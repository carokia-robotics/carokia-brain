# Demo Script: Chat CLI

## Overview
Demonstrate Carokia's conversational ability using a local LLM via Ollama. This is the simplest entry point -- no hardware, no special features, just a terminal.

## Prerequisites
- Ollama running with `gemma3:latest` (or any model configured in `carokia.toml`)

## Commands

```bash
# Start Ollama if not already running
ollama serve &

# Run the chat demo
cargo run --example chat_cli
```

## Walkthrough

1. **Launch** -- run the command above. The banner appears:
   ```
   +===================================+
   |     CAROKIA -- AI Companion       |
   |     Type 'quit' to exit           |
   +===================================+
   ```

2. **First message** -- type: `Hello, what are you?`
   - Expected: Carokia introduces itself as an autonomous robot companion.

3. **Follow-up** -- type: `What can you help me with?`
   - Expected: Carokia describes its capabilities (navigation, perception, protection).

4. **Context test** -- type: `Remember that my name is Alex.` then ask `What's my name?`
   - Expected: Carokia recalls "Alex" within the same session (conversation history).

5. **Exit** -- type `quit`.

## Talking Points

- Carokia uses Ollama for fully local, private inference -- no data leaves your machine.
- The conversation manager maintains a sliding window of context (default 20 messages).
- The LLM backend is trait-based: swap Ollama for Claude or any other provider by changing config.
- Response quality depends on the model you pull -- larger models give better results but need more RAM.
