---
title: Error Codes Reference
sidebar_position: 10
---

# Error Codes Reference

This page documents all error codes, messages, and their solutions in Ahnlich DB and AI.

## gRPC Status Codes

Ahnlich uses standard gRPC status codes for error responses:

| gRPC Code | HTTP Code | When Used |
|-----------|-----------|-----------|
| `NotFound` | 404 | Store, predicate, or index not found |
| `AlreadyExists` | 409 | Store already exists |
| `InvalidArgument` | 400 | Invalid input, dimension mismatch, type errors |
| `ResourceExhausted` | 429 | Memory allocation failures |
| `FailedPrecondition` | 400 | Missing dependencies or prerequisites |
| `OutOfRange` | 400 | Token limits, invalid ranges |
| `Internal` | 500 | Model errors, runtime failures |

---

## Database (ahnlich-db) Errors

### Store Errors

#### StoreNotFound

**Error Message:** `Store "store_name" not found`

**gRPC Code:** `NotFound`

**Cause:** Attempting to access a store that doesn't exist.

**Solution:**
- Create the store first using `CREATESTORE`
- Verify store name spelling
- Use `LISTSTORES` to check available stores
- Check if persistence file loaded correctly on restart

**Example:**
```
LISTSTORES
CREATESTORE mystore DIMENSION 128
```

---

#### StoreAlreadyExists

**Error Message:** `Store "store_name" already exists`

**gRPC Code:** `AlreadyExists`

**Cause:** Creating a store when one with that name already exists and `error_if_exists=true`.

**Solution:**
- Use a different store name
- Set `error_if_exists=false` to silently skip creation
- Drop the existing store first if you want to recreate it

**Example:**
```rust
CreateStore {
    store: "my_store".to_string(),
    dimension: 128,
    create_predicates: vec![],
    non_linear_indices: vec![],
    error_if_exists: false,  // Won't error if exists
}
```

---

#### StoreDimensionMismatch

**Error Message:** `Store dimension is [128], input dimension of [256] was specified`

**gRPC Code:** `InvalidArgument`

**Cause:** Vector dimensions don't match the store's configured dimension.

**Solution:**
- Ensure all vectors match store dimension
- Check embedding model output dimensions
- For AI stores, verify query and index models have same embedding size

**Common Model Dimensions:**
- `all-minilm-l6-v2`: 384
- `all-minilm-l12-v2`: 384
- `bge-base-en-v1.5`: 768
- `bge-large-en-v1.5`: 1024
- `resnet-50`: 2048
- `clip-vit-b32-*`: 512
- `clap-audio` / `clap-text`: 512
- `buffalo-l`: 512
- `sface-yunet`: 128

---

### Index Errors

#### PredicateNotFound

**Error Message:** `Predicate "field_name" not found in store, attempt CREATEPREDINDEX with predicate`

**gRPC Code:** `NotFound`

**Cause:** Querying using a predicate that hasn't been indexed.

**Solution:**
```
CREATEPREDINDEX my_store PREDICATES (field_name)
```

Or include predicates when creating store:
```
CREATESTORE my_store DIMENSION 128 PREDICATES (author, category)
```

---

#### NonLinearIndexNotFound

**Error Message:** `Non linear algorithm KDTree not found in store, create store with support`

**gRPC Code:** `NotFound`

**Cause:** Attempting to use a non-linear algorithm (KDTree) not created with the store.

**Solution:**
```
CREATE_NON_LINEAR_ALGORITHM_INDEX my_store NONLINEARALGORITHMINDEX (KDTree)
```

Or include when creating store:
```
CREATESTORE my_store DIMENSION 128 NONLINEARALGORITHMINDEX (KDTree)
```

---

### Resource Errors

#### Allocation

**Error Message:** `allocation error: CapacityOverflow`

**gRPC Code:** `ResourceExhausted`

**Cause:** Memory allocation failed - hit the `--allocator-size` limit.

**Solution:**
- Increase `--allocator-size` when starting server
- Reduce batch sizes
- Monitor memory usage
- For images, use `--enable-streaming` in AI proxy

**Example:**
```bash
ahnlich-db run --allocator-size 21474836480  # 20 GiB
```

---

## AI Proxy (ahnlich-ai) Errors

### Store Errors

#### StoreNotFound / StoreAlreadyExists

Same as DB errors above, but for AI stores.

---

#### StoreTypeMismatchError

**Error Message:** `Cannot query Input. Store expects [RawString], input type [Image] was provided`

**gRPC Code:** `InvalidArgument`

**Cause:** Sending wrong input type (text to image model or vice versa).

**Solution:**
- Use text (RawString) for text models: `all-minilm-*`, `bge-*`, `clip-vit-b32-text`
- Use images (Image bytes) for image models: `resnet-50`, `clip-vit-b32-image`

---

### Input Errors

#### InputNotSpecified

**Error Message:** `"search_input" input not specified`

**gRPC Code:** `InvalidArgument`

**Cause:** Required input field is missing or empty.

**Solution:** Provide the required input (text or image).

---

#### TokenExceededError

**Error Message:** `Max Token Exceeded. Model Expects [256], input type was [512]`

**gRPC Code:** `OutOfRange`

**Cause:** Input text exceeds model's maximum token limit.

**Token Limits:**
- `all-minilm-l6-v2`: 256 tokens
- `all-minilm-l12-v2`: 256 tokens
- `bge-base-en-v1.5`: 512 tokens
- `bge-large-en-v1.5`: 512 tokens
- `clip-vit-b32-text`: 77 tokens

**Solution:**
- Truncate text to fit within limit
- Split long documents into chunks
- Use model with larger token limit (BGE models support 512 tokens)

**Example:**
```python
# Truncate text
text = long_text[:500]  # Rough approximation

# Or split into chunks
chunks = [text[i:i+500] for i in range(0, len(text), 500)]
```

---

#### ImageDimensionsMismatchError

**Error Message:** `Image Dimensions [(512, 512)] does not match the expected model dimensions [(224, 224)]`

**gRPC Code:** `InvalidArgument`

**Cause:** Image size doesn't match model requirements.

**Model Requirements:**
- `resnet-50`: 224x224 pixels
- `clip-vit-b32-image`: 224x224 pixels

**Solution:**
- Resize images to 224x224 before sending
- Use `PreprocessAction.ModelPreprocessing` to auto-resize

**Example (Python):**
```python
from PIL import Image

img = Image.open("photo.jpg")
img = img.resize((224, 224))
```

---

#### ReservedError

**Error Message:** `Reserved key "_ahnlich_input_key" used`

**gRPC Code:** `InvalidArgument`

**Cause:** Using reserved metadata key `_ahnlich_input_key`.

**Solution:** Avoid using `_ahnlich_input_key` in your metadata - this key is reserved for internal use when `store_original=true`.

---

### Model Errors

#### AIModelNotInitialized

**Error Message:** `index_model or query_model not selected or loaded during aiproxy startup`

**gRPC Code:** `Internal`

**Cause:** Models not loaded at AI proxy startup.

**Solution:**
- Ensure models are specified in `--supported-models`
- Check model cache location has write permissions
- Verify network connectivity for model downloads (first use)
- Check disk space in model cache directory

**Example:**
```bash
ahnlich-ai run \
  --supported-models all-minilm-l6-v2,resnet-50 \
  --model-cache-location /path/to/models
```

---

#### AIModelNotSupported

**Error Message:** `index_model or query_model "model_name" not supported`

**gRPC Code:** `Internal`

**Cause:** Using a model not in the supported models list.

**Supported Models:**
- `all-minilm-l6-v2` (Text, 384-dim)
- `all-minilm-l12-v2` (Text, 384-dim)
- `bge-base-en-v1.5` (Text, 768-dim)
- `bge-large-en-v1.5` (Text, 1024-dim)
- `resnet-50` (Image, 2048-dim)
- `clip-vit-b32-image` (Image, 512-dim)
- `clip-vit-b32-text` (Text, 512-dim)
- `clap-audio` (Audio, 512-dim)
- `clap-text` (Text, 512-dim)
- `buffalo-l` (Face, 512-dim)
- `sface-yunet` (Face, 128-dim)

**Solution:** Use one of the supported models above.

---

#### DimensionsMismatchError

**Error Message:** `Dimensions Mismatch between index [768], and Query [1024] Models`

**gRPC Code:** `InvalidArgument`

**Cause:** Index and query models have different embedding dimensions.

**Solution:** Use compatible models with same embedding dimensions:
- AllMiniLM (L6/L12): both 384-dim
- BGE-Base: 768-dim
- BGE-Large: 1024-dim
- ClipVit-B32: both (text/image) 512-dim
- CLAP: both (audio/text) 512-dim
- Buffalo_L: 512-dim
- SFace+YuNet: 128-dim

---

#### ModelInitializationError

**Error Message:** `Error initializing a model thread: failed to download model`

**gRPC Code:** `Internal`

**Cause:** Model initialization failed during download or loading.

**Solutions:**
- Check internet connectivity (models download from HuggingFace on first use)
- Verify disk space in model cache location
- Check firewall allows HuggingFace Hub access
- Clear corrupted model cache and retry

**Cache Location:** Default `~/.ahnlich/models`

---

### Database Connection Errors

#### DatabaseClientError

**Error Message:** `Proxy Errored with connection refused`

**gRPC Code:** `FailedPrecondition`

**Cause:** AI proxy cannot connect to database.

**Solution:**
- Ensure database is running before starting AI proxy
- Verify `--db-host` and `--db-port` settings
- Check firewall rules
- For standalone mode, use `--without-db` flag

**Example:**
```bash
# Start DB first
ahnlich-db run --port 1369

# Then start AI
ahnlich-ai run --db-host 127.0.0.1 --db-port 1369
```

---

### Operation Errors

#### DelKeyError

**Error Message:** `Cannot call DelKey on store with store_original as false`

**gRPC Code:** `FailedPrecondition`

**Cause:** Attempting to delete keys when `store_original=false`.

**Solution:** Recreate store with `store_original=true` if you need to delete original inputs:

```python
CreateStore(
    store="my_store",
    query_model=AiModel.ALL_MINI_LM_L6_V2,
    index_model=AiModel.ALL_MINI_LM_L6_V2,
    store_original=True,  # Required for DelKey
)
```

---

### Image Processing Errors

#### ImageBytesDecodeError

**Error Message:** `Bytes could not be successfully decoded into an image`

**gRPC Code:** `Internal`

**Cause:** Invalid or corrupted image bytes.

**Solution:**
- Use standard image formats: JPEG, PNG, BMP, GIF
- Validate image files aren't corrupted
- Ensure proper encoding if using base64

---

#### ImageNonzeroDimensionError

**Error Message:** `Image can't have zero value in any dimension. Found height: 0, width: 100`

**gRPC Code:** `Internal`

**Cause:** Image has zero width or height.

**Solution:** Validate image dimensions before sending - both width and height must be > 0.

---

### Audio Processing Errors

#### AudioBytesDecodeError

**Error Message:** `Bytes could not be successfully decoded into audio: <details>`

**gRPC Code:** `Internal`

**Cause:** Invalid or corrupted audio bytes, or unsupported audio format.

**Solution:**
- Use supported audio formats: WAV, MP3, FLAC, OGG
- Validate audio files aren't corrupted
- Ensure proper encoding

---

#### AudioTooLongError

**Error Message:** `Audio input is too long (15000ms). Model accepts at most 10000ms per clip. Trim or split your audio before indexing.`

**gRPC Code:** `InvalidArgument`

**Cause:** Audio clip exceeds the 10-second maximum duration for CLAP models.

**Solution:**
- Trim audio to 10 seconds or less
- Split longer audio into multiple clips
- Use audio editing tools to extract relevant segments

**Example (Python with pydub):**
```python
from pydub import AudioSegment

audio = AudioSegment.from_file("long_audio.wav")
# Take first 10 seconds
clip = audio[:10000]  # milliseconds
clip.export("short_clip.wav", format="wav")
```

---

#### AudioNoPreprocessingError

**Error Message:** `NoPreprocessing is not supported for audio inputs. Audio requires decoding, resampling, and log-Mel spectrogram conversion before it can be passed to the model.`

**gRPC Code:** `InvalidArgument`

**Cause:** Attempted to use `NoPreprocessing` with audio input.

**Solution:** Always use `ModelPreprocessing` for audio inputs:
```python
Set(
    store="audio_store",
    inputs=[...],
    preprocess_action=PreprocessAction.ModelPreprocessing,  # Required for audio
)
```

---

#### AudioResampleError

**Error Message:** `Audio could not be resampled: <details>`

**gRPC Code:** `Internal`

**Cause:** Failed to resample audio to required 48kHz sample rate.

**Solution:**
- Pre-convert audio to 48kHz before sending
- Ensure audio file is not corrupted

---

### Face Recognition Errors

#### FaceModelNoPreprocessingError

**Error Message:** `NoPreprocessing is not supported for face recognition models. Face models require multi-stage detection and alignment that cannot be bypassed.`

**gRPC Code:** `InvalidArgument`

**Cause:** Attempted to use `NoPreprocessing` with face detection models (Buffalo_L or SFace+YuNet).

**Solution:** Always use `ModelPreprocessing` for face models:
```python
Set(
    store="faces_store",
    inputs=[...],
    preprocess_action=PreprocessAction.ModelPreprocessing,  # Required for face models
)
```

---

#### MultipleEmbeddingsForQuery

**Error Message:** `Query input produced 3 embeddings - query input must produce exactly 1 embedding`

**gRPC Code:** `InvalidArgument`

**Cause:** Face detection found multiple faces in a query image, but queries must produce exactly one embedding.

**Solution:**
- For queries, use images with exactly one face
- Crop the image to contain only the target face
- For indexing multiple faces, use `Set` (which supports multiple embeddings per image)

---

## Client Errors

### InvalidURI

**Error Message:** `Invalid URI "invalid-uri"`

**gRPC Code:** `InvalidArgument`

**Cause:** Malformed connection URI.

**Solution:** Use valid URI format:
```
http://127.0.0.1:1369  (DB)
http://127.0.0.1:1370  (AI)
```

---

### Tonic (Transport Error)

**Error Message:** `Transport issues with tonic: connection error`

**gRPC Code:** `Internal`

**Cause:** Network or transport layer error.

**Solutions:**
- Check server is running and accessible
- Verify network connectivity
- Check firewall rules
- Ensure correct host/port configuration

---

## DSL/CLI Errors

### UnsupportedAlgorithm

**Error Message:** `Found unsupported algorithm "invalid_algo"`

**Cause:** Unknown algorithm name in DSL command.

**Valid Algorithms:**
- `EuclideanDistance`
- `DotProductSimilarity`
- `CosineSimilarity`
- `KDTree`
- `HNSW`

---

### UnsupportedAIModel

**Error Message:** `Found unsupported ai model "invalid_model"`

**Cause:** Unknown AI model name.

**Solution:** Use one of the 11 supported models:
- Text: `all-minilm-l6-v2`, `all-minilm-l12-v2`, `bge-base-en-v1.5`, `bge-large-en-v1.5`, `clip-vit-b32-text`, `clap-text`
- Image: `resnet-50`, `clip-vit-b32-image`
- Face: `buffalo-l`, `sface-yunet`
- Audio: `clap-audio`

---

### UnsupportedPreprocessingMode

**Error Message:** `Unexpected preprocessing mode`

**Cause:** Invalid preprocessing option.

**Valid Options:**
- `NoPreprocessing`
- `ModelPreprocessing`

---

## Common Error Combinations

### "Connection Refused" + "StoreNotFound"

Indicates server connectivity issues. Check:
1. Server is running
2. Host/port configuration is correct
3. Firewall allows connections

### "DimensionMismatch" + AI Store

Check that query_model and index_model have same embedding dimensions.

### "TokenExceeded" + Long Text

Split text into smaller chunks or use model with larger token limit.

---

## Getting Help

If you encounter an error not listed here:

1. Check server logs for detailed stack traces
2. Enable tracing: `--enable-tracing --otel-endpoint http://jaeger:4317`
3. Report issues: [GitHub Issues](https://github.com/deven96/ahnlich/issues)
4. Ask in: [WhatsApp Community](https://chat.whatsapp.com/E4CP7VZ1lNH9dJUxpsZVvD)
