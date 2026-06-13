# audio

### Purpose

Audio processing APIs — ASR, TTS, classification, and audio-to-audio transformation.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — `Audio<E>` type and submodule declarations |
| `types.rs` | Shared audio types — `AudioInput`, `AudioOutput` |
| `asr.rs` | Automatic speech recognition (transcription) |
| `tts.rs` | Text-to-speech generation |
| `classification.rs` | Audio classification with labels |
| `audio_to_audio.rs` | Audio-to-audio transformation (noise reduction, enhancement) |
