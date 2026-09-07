# Audio Search Example

Find similar songs using audio samples. Works like Shazam.

Uses CLAP model for audio embeddings.

## Features

- Find similar songs
- Supports audio up to 10 minutes
- Record from microphone and search

## Prerequisites

- Go 1.21+
- Ahnlich DB and AI servers running locally

## Setup

### 1. Start Ahnlich

Use Docker:

```bash
docker run -d --name ahnlich-db -p 1369:1369 ghcr.io/deven96/ahnlich-db:latest
docker run -d --name ahnlich-ai -p 1370:1370 \
  ghcr.io/deven96/ahnlich-ai:latest \
  ahnlich-ai run --db-host host.docker.internal --supported-models clap-audio
```

### 2. Install Dependencies

```bash
go mod download
```

### 3. Add Songs

Copy your music to `audio/songs/`:

```bash
cp ~/Music/*.mp3 audio/songs/
```

Supports: MP3, WAV, OGG, FLAC, M4A

**Or download royalty-free music:**
- [Free Music Archive](https://freemusicarchive.org/) - CC0/CC-BY tracks
- [YouTube Audio Library](https://www.youtube.com/audiolibrary) - Royalty-free music
- [ccMixter](http://ccmixter.org/) - Creative Commons licensed

## Usage

### Index Songs

```bash
go run main.go index --dir ./audio/songs
```

Creates embeddings for all songs in directory.

### Search for Similar Songs

```bash
go run main.go search --file ./audio/songs/your_song.mp3 --limit 5
```

Finds top 5 similar songs.

Add `--play` to play the matched section:
```bash
go run main.go search --file query.mp3 --limit 5 --play
```

### Record and Search

```bash
go run main.go record --duration 10 --limit 5
```

Records from microphone, searches, and plays the matched section.

Requires `ffmpeg`:
```bash
brew install ffmpeg
```

## Example Output

```
Indexing audio files...

✓ Indexed: rain.ogg (category: nature)
✓ Indexed: cat_meow.ogg (category: animal)
✓ Indexed: dog_bark.ogg (category: animal)

Total: 3 audio files indexed

---

Searching for audio similar to: cat_meow.ogg

Results:
1. cat_meow.ogg (similarity: 1.000, category: animal) ← exact match
2. dog_bark.ogg (similarity: 0.782, category: animal) ← similar animal sound
3. rain.ogg (similarity: 0.234, category: nature)
```

## How It Works

Audio files become 512-dimensional vectors.

### Chunking

Long audio (>10s) splits into chunks:
- 10-second chunks
- 1-second overlap
- Each chunk gets metadata

### Similarity

Similar sounds have similar embeddings.

Uses cosine similarity to compare.

## License

Apache 2.0
