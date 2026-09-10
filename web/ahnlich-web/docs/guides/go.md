---
title: Go Audio Search
---

**Source**: [examples/go/audio-search](https://github.com/deven96/ahnlich/tree/main/examples/go/audio-search)

This guide demonstrates building an **audio similarity search** application using the Go SDK. It covers:

- Creating AI stores for audio embeddings using CLAP
- Indexing audio files (MP3, WAV, OGG, FLAC)
- Recording from microphone and searching for similar songs
- Audio chunking for long files (up to 10 minutes)
- Using metadata for filtering and organizing results

## 🔧 What you'll learn

1. Setting up an **AI Store** for audio using the Go client
2. Indexing music files and generating audio embeddings
3. Recording live audio from microphone
4. Searching for similar songs (like Shazam)
5. Understanding audio chunking for long audio files
6. Playing back matched sections

## 💡 Highlighted snippet

```go
package main

import (
    "context"
    "google.golang.org/grpc"
    aimodel "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
    aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
)

// Connect to AI server
conn, _ := grpc.DialContext(ctx, "localhost:1370",
    grpc.WithTransportCredentials(insecure.NewCredentials()))
client := aisvc.NewAIServiceClient(conn)

// Create store for audio search
client.CreateStore(ctx, &aiquery.CreateStore{
    Store:         "song_library",
    QueryModel:    aimodel.AIModel_CLAP_AUDIO,
    IndexModel:    aimodel.AIModel_CLAP_AUDIO,
    ErrorIfExists: false,
    StoreOriginal: false,
    Predicates:    []string{"filename"},
})

// Index an audio file
audioData, _ := os.ReadFile("song.mp3")
client.Set(ctx, &aiquery.Set{
    Store: "song_library",
    Inputs: []*keyval.AiStoreEntry{
        {
            Key: &keyval.StoreInput{
                Value: &keyval.StoreInput_Audio{Audio: audioData},
            },
            Value: &keyval.StoreValue{
                Value: map[string]*metadata.MetadataValue{
                    "filename": {Value: &metadata.MetadataValue_RawString{
                        RawString: "song.mp3",
                    }},
                },
            },
        },
    },
    PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
})

// Search for similar audio
resp, _ := client.GetSimN(ctx, &aiquery.GetSimN{
    Store: "song_library",
    SearchInput: &keyval.StoreInput{
        Value: &keyval.StoreInput_Audio{Audio: queryAudio},
    },
    ClosestN:         5,
    Algorithm:        algorithms.Algorithm_CosineSimilarity,
    PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
})
```

## 🎵 Features

- **Index songs**: Process entire music libraries (up to 10 minutes per file)
- **Record & search**: Record from microphone and identify songs (10-second limit)
- **Audio chunking**: Automatically handles long audio during indexing (10-second chunks with 1-second overlap)
- **Query trimming**: Search queries automatically use first 10 seconds
- **Cross-modal search**: Search audio using text descriptions
- **Playback**: Play matched songs from the beginning

## ➕ Try it yourself

- Clone the example repository
- Launch `ahnlich-db` and `ahnlich-ai` locally via Docker
- Add your music files to `audio/songs/`
- Run `go run main.go index --dir ./audio/songs`
- Test recording: `go run main.go record --duration 10`
- Play a song from speakers and watch it match!
