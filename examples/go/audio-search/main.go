package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	aimodel "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
	aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/algorithms"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
	aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
)

const (
	defaultAIAddr = "localhost:1370"
	storeName     = "song_library"
)

func main() {
	indexCmd := flag.NewFlagSet("index", flag.ExitOnError)
	audioDir := indexCmd.String("dir", "./audio/songs", "directory with audio files")

	searchCmd := flag.NewFlagSet("search", flag.ExitOnError)
	queryFile := searchCmd.String("file", "", "audio file to search")
	limit := searchCmd.Int("limit", 5, "results to return")
	play := searchCmd.Bool("play", false, "play the matched section")

	recordCmd := flag.NewFlagSet("record", flag.ExitOnError)
	recordLimit := recordCmd.Int("limit", 5, "results to return")
	recordDuration := recordCmd.Int("duration", 10, "seconds (max 10)")
	recordPlay := recordCmd.Bool("play", true, "play the matched section")

	if len(os.Args) < 2 {
		fmt.Println("Usage:")
		fmt.Println("  go run main.go index --dir <audio_directory>")
		fmt.Println("  go run main.go search --file <query_audio.mp3> --limit 5")
		fmt.Println("  go run main.go record --duration 10 --limit 5")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "index":
		indexCmd.Parse(os.Args[2:])
		if err := indexAudio(*audioDir); err != nil {
			log.Fatalf("Index failed: %v", err)
		}
	case "search":
		searchCmd.Parse(os.Args[2:])
		if *queryFile == "" {
			log.Fatal("--file required")
		}
		if err := searchAudio(*queryFile, *limit, *play); err != nil {
			log.Fatalf("Search failed: %v", err)
		}
	case "record":
		recordCmd.Parse(os.Args[2:])
		if *recordDuration > 10 || *recordDuration < 1 {
			log.Fatal("duration must be 1-10 seconds")
		}
		if err := recordAndSearch(*recordDuration, *recordLimit, *recordPlay); err != nil {
			log.Fatalf("Record failed: %v", err)
		}
	default:
		fmt.Printf("Unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}

func connect() (*grpc.ClientConn, aisvc.AIServiceClient, context.Context, context.CancelFunc, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	conn, err := grpc.DialContext(ctx, defaultAIAddr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(100*1024*1024), // 100MB
			grpc.MaxCallSendMsgSize(100*1024*1024), // 100MB
		))
	if err != nil {
		cancel()
		return nil, nil, nil, nil, fmt.Errorf("connect to %s failed: %w", defaultAIAddr, err)
	}

	client := aisvc.NewAIServiceClient(conn)
	return conn, client, ctx, cancel, nil
}

func indexAudio(dir string) error {
	conn, client, ctx, cancel, err := connect()
	if err != nil {
		return err
	}
	defer cancel()
	defer conn.Close()

	fmt.Println("Creating store...")
	_, err = client.CreateStore(ctx, &aiquery.CreateStore{
		Store:         storeName,
		QueryModel:    aimodel.AIModel_CLAP_AUDIO,
		IndexModel:    aimodel.AIModel_CLAP_AUDIO,
		ErrorIfExists: false,
		StoreOriginal: false,
		Predicates:    []string{"filename"},
	})
	if err != nil {
		return fmt.Errorf("create store failed: %w", err)
	}

	files, err := findAudioFiles(dir)
	if err != nil {
		return err
	}

	if len(files) == 0 {
		return fmt.Errorf("no audio in %s (supports: .mp3, .wav, .ogg, .flac)", dir)
	}

	fmt.Printf("\nIndexing %d files from: %s\n\n", len(files), dir)

	for i, file := range files {
		if err := indexFile(ctx, client, file); err != nil {
			log.Printf("⚠ Failed %s: %v", filepath.Base(file), err)
			continue
		}
		fmt.Printf("✓ [%d/%d] %s\n", i+1, len(files), filepath.Base(file))
	}

	fmt.Printf("\n✅ Indexed %d songs\n", len(files))
	return nil
}

func findAudioFiles(dir string) ([]string, error) {
	var files []string
	exts := map[string]bool{".mp3": true, ".wav": true, ".ogg": true, ".flac": true, ".m4a": true}

	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && exts[filepath.Ext(path)] {
			files = append(files, path)
		}
		return nil
	})

	return files, err
}

func indexFile(ctx context.Context, client aisvc.AIServiceClient, file string) error {
	audioData, err := os.ReadFile(file)
	if err != nil {
		return err
	}

	_, err = client.Set(ctx, &aiquery.Set{
		Store: storeName,
		Inputs: []*keyval.AiStoreEntry{
			{
				Key: &keyval.StoreInput{
					Value: &keyval.StoreInput_Audio{Audio: audioData},
				},
				Value: &keyval.StoreValue{
					Value: map[string]*metadata.MetadataValue{
						"filename": {Value: &metadata.MetadataValue_RawString{
							RawString: filepath.Base(file),
						}},
					},
				},
			},
		},
		PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
	})

	return err
}

func searchAudio(queryFile string, limit int, playAudio bool) error {
	conn, client, ctx, cancel, err := connect()
	if err != nil {
		return err
	}
	defer cancel()
	defer conn.Close()

	audioData, err := os.ReadFile(queryFile)
	if err != nil {
		return fmt.Errorf("read query failed: %w", err)
	}

	fmt.Printf("Searching for: %s\n\n", filepath.Base(queryFile))

	resp, err := client.GetSimN(ctx, &aiquery.GetSimN{
		Store: storeName,
		SearchInput: &keyval.StoreInput{
			Value: &keyval.StoreInput_Audio{Audio: audioData},
		},
		ClosestN:         uint64(limit),
		Algorithm:        algorithms.Algorithm_CosineSimilarity,
		PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
	})
	if err != nil {
		return fmt.Errorf("search failed: %w", err)
	}

	if len(resp.Entries) == 0 {
		fmt.Println("No results. Run: go run main.go index")
		return nil
	}

	fmt.Println("Results:")
	fmt.Println("----------------------------------------")
	
	songScores := make(map[string]float32)
	songCounts := make(map[string]int)
	
	for i, entry := range resp.Entries {
		filename := "unknown"
		var chunkStartSec, chunkEndSec string
		
		if entry.Value != nil && entry.Value.Value != nil {
			if meta := entry.Value.Value["filename"]; meta != nil {
				filename = meta.GetRawString()
			}
			if chunkStart := entry.Value.Value["chunk_start_sec"]; chunkStart != nil {
				chunkStartSec = chunkStart.GetRawString()
				chunkEndSec = entry.Value.Value["chunk_end_sec"].GetRawString()
			}
		}

		similarity := float32(0.0)
		if entry.Similarity != nil {
			similarity = entry.Similarity.Value
		}
		
		songScores[filename] += similarity
		songCounts[filename]++
		
		fmt.Printf("%d. %s\n", i+1, filename)
		fmt.Printf("   Similarity: %.3f\n", similarity)

		if chunkStartSec != "" {
			totalChunks := entry.Value.Value["total_chunks"].GetRawString()
			fmt.Printf("   Chunk: %s-%ss (of %s)\n", chunkStartSec, chunkEndSec, totalChunks)
		}
		fmt.Println()
	}

	if playAudio {
		var bestSong string
		var bestAvg float32 = 0.0
		
		for song, totalScore := range songScores {
			avg := totalScore / float32(songCounts[song])
			if avg > bestAvg {
				bestAvg = avg
				bestSong = song
			}
		}
		
		if bestSong != "" {
			fmt.Printf("\n🎯 Best match (avg %.3f): %s\n", bestAvg, bestSong)
			return playMatchedSection(&matchInfo{filename: bestSong})
		}
	}

	return nil
}

type matchInfo struct {
	filename      string
	chunkStartSec string
	chunkEndSec   string
}

func playMatchedSection(match *matchInfo) error {
	audioDir := "./audio/songs"
	fullPath := filepath.Join(audioDir, match.filename)
	
	if _, err := os.Stat(fullPath); os.IsNotExist(err) {
		return fmt.Errorf("audio file not found: %s", fullPath)
	}

	fmt.Printf("🔊 Playing: %s (from beginning)\n", match.filename)

	cmd := exec.Command("ffplay",
		"-nodisp",
		"-autoexit",
		fullPath,
	)

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("playback failed (ffmpeg installed?): %w", err)
	}

	return nil
}

func recordAndSearch(duration, limit int, playAudio bool) error {
	fmt.Printf("🎤 Recording %d seconds...\n", duration)
	fmt.Println("Press ENTER to start, or Ctrl+C to cancel")
	fmt.Scanln()

	rawFile := "recorded_raw.wav"
	outputFile := "recorded_query.wav"
	fmt.Printf("🔴 Recording... (%ds)\n", duration)

	cmd := exec.Command("ffmpeg",
		"-f", "avfoundation",
		"-i", ":0",
		"-t", fmt.Sprintf("%d", duration),
		"-y",
		rawFile,
	)

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("record failed (ffmpeg installed?): %w\nTry: brew install ffmpeg", err)
	}

	fmt.Println("🧹 Normalizing audio...")

	cleanCmd := exec.Command("ffmpeg",
		"-i", rawFile,
		"-af", "loudnorm",
		"-y",
		outputFile,
	)

	if err := cleanCmd.Run(); err != nil {
		return fmt.Errorf("audio normalization failed: %w", err)
	}

	fmt.Println("✅ Recording complete!\n")

	return searchAudio(outputFile, limit, playAudio)
}
