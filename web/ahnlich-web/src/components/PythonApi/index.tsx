import {useState, type ReactNode} from 'react';
import CodeBlock from '@theme/CodeBlock';

const STEP_META = [
  {
    label: 'Connect',
    blurb: 'Open a client to the Ahnlich AI proxy. No boilerplate, no config.',
  },
  {
    label: 'Create a store',
    blurb: 'Pick an embedding model and let Ahnlich handle the vectors for you.',
  },
  {
    label: 'Insert data',
    blurb: 'Send raw text with metadata. Embeddings are generated automatically.',
  },
  {
    label: 'Search',
    blurb: 'Query by meaning, with results ranked by similarity.',
  },
];

type Sdk = {
  label: string;
  lang: string;
  ext: string;
  install: string;
  code: [string, string, string, string];
};

const SDKS: Sdk[] = [
  {
    label: 'Python',
    lang: 'python',
    ext: 'py',
    install: 'pip install ahnlich-client-py',
    code: [
      `import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.ai_service import AiServiceStub


async def main():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        # ready to talk to Ahnlich`,
      `from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel

await client.create_store(ai_query.CreateStore(
    store="books",
    index_model=AiModel.ALL_MINI_LM_L6_V2,
    query_model=AiModel.ALL_MINI_LM_L6_V2,
    predicates=["author", "genre"],
    store_original=True,
    error_if_exists=True,
))`,
      `from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.ai import preprocess

await client.set(ai_query.Set(
    store="books",
    inputs=[
        keyval.AiStoreEntry(
            key=keyval.StoreInput(raw_string="A galactic empire in decline..."),
            value=keyval.StoreValue(value={
                "genre": metadata.MetadataValue(raw_string="SciFi"),
                "author": metadata.MetadataValue(raw_string="Asimov"),
            }),
        )
    ],
    preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
))`,
      `from ahnlich_client_py.grpc.algorithm import algorithms
from ahnlich_client_py.grpc.ai.preprocess import PreprocessAction

response = await client.get_sim_n(ai_query.GetSimN(
    store="books",
    search_input=keyval.StoreInput(raw_string="space opera classics"),
    closest_n=3,
    algorithm=algorithms.Algorithm.CosineSimilarity,
    preprocess_action=PreprocessAction.ModelPreprocessing,
))

for entry in response.entries:
    print(entry.key.raw_string, entry.similarity.value)`,
    ],
  },
  {
    label: 'Rust',
    lang: 'rust',
    ext: 'rs',
    install: 'cargo add ahnlich_client_rs',
    code: [
      `use ahnlich_client_rs::ai::AiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AiClient::new("127.0.0.1:1370".to_string()).await?;
    // ready to talk to Ahnlich
    Ok(())
}`,
      `use ahnlich_types::ai::query::CreateStore;
use ahnlich_types::ai::models::AiModel;

client.create_store(CreateStore {
    store: "books".to_string(),
    schema: None,
    index_model: AiModel::AllMiniLmL6V2 as i32,
    query_model: AiModel::AllMiniLmL6V2 as i32,
    predicates: vec!["author".into(), "genre".into()],
    non_linear_indices: vec![],
    error_if_exists: true,
    store_original: true,
}, None).await?;`,
      `use ahnlich_types::ai::query::Set;
use ahnlich_types::ai::preprocess::PreprocessAction;
use ahnlich_types::keyval::{AiStoreEntry, StoreInput, StoreValue};
use ahnlich_types::keyval::store_input::Value;
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MValue};
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("genre".to_string(),
    MetadataValue { value: Some(MValue::RawString("SciFi".into())) });

client.set(Set {
    store: "books".to_string(),
    schema: None,
    execution_provider: None,
    preprocess_action: PreprocessAction::ModelPreprocessing as i32,
    inputs: vec![AiStoreEntry {
        key: Some(StoreInput { value: Some(Value::RawString("A galactic empire in decline...".into())) }),
        value: Some(StoreValue { value: metadata }),
    }],
    model_params: HashMap::new(),
}, None).await?;`,
      `use ahnlich_types::ai::query::GetSimN;
use ahnlich_types::algorithm::algorithms::Algorithm;

let res = client.get_sim_n(GetSimN {
    store: "books".to_string(),
    schema: None,
    search_input: Some(StoreInput {
        value: Some(Value::RawString("space opera classics".into())),
    }),
    closest_n: 3,
    algorithm: Algorithm::CosineSimilarity as i32,
    execution_provider: None,
    preprocess_action: PreprocessAction::ModelPreprocessing as i32,
    condition: None,
    model_params: HashMap::new(),
}, None).await?;

println!("{:?}", res.entries);`,
    ],
  },
  {
    label: 'Node',
    lang: 'typescript',
    ext: 'ts',
    install: 'npm install ahnlich-client-node',
    code: [
      `import { createAiClient } from "ahnlich-client-node";

const client = createAiClient("127.0.0.1:1370");
// ready to talk to Ahnlich`,
      `import { CreateStore } from "ahnlich-client-node/grpc/ai/query_pb";
import { AIModel } from "ahnlich-client-node/grpc/ai/models_pb";

await client.createStore(
  new CreateStore({
    store: "books",
    queryModel: AIModel.ALL_MINI_LM_L6_V2,
    indexModel: AIModel.ALL_MINI_LM_L6_V2,
    predicates: ["author", "genre"],
    errorIfExists: true,
    storeOriginal: true,
  })
);`,
      `import { Set } from "ahnlich-client-node/grpc/ai/query_pb";
import { AiStoreEntry, StoreInput, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

await client.set(
  new Set({
    store: "books",
    inputs: [
      new AiStoreEntry({
        key: new StoreInput({ value: { case: "rawString", value: "A galactic empire in decline..." } }),
        value: new StoreValue({
          value: {
            genre: new MetadataValue({ value: { case: "rawString", value: "SciFi" } }),
            author: new MetadataValue({ value: { case: "rawString", value: "Asimov" } }),
          },
        }),
      }),
    ],
    preprocessAction: PreprocessAction.MODEL_PREPROCESSING,
  })
);`,
      `import { GetSimN } from "ahnlich-client-node/grpc/ai/query_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

const response = await client.getSimN(
  new GetSimN({
    store: "books",
    searchInput: new StoreInput({ value: { case: "rawString", value: "space opera classics" } }),
    closestN: 3,
    algorithm: Algorithm.COSINE_SIMILARITY,
  })
);

for (const entry of response.entries) {
  console.log(entry.input?.value, entry.similarity);
}`,
    ],
  },
  {
    label: 'Go',
    lang: 'go',
    ext: 'go',
    install: 'go get github.com/deven96/ahnlich/sdk/ahnlich-client-go',
    code: [
      `import (
    "context"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
)

conn, err := grpc.DialContext(ctx, "127.0.0.1:1370",
    grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
if err != nil {
    log.Fatalf("failed to connect: %v", err)
}
defer conn.Close()

client := aisvc.NewAIServiceClient(conn)`,
      `import (
    aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    aimodel "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
)

_, err = client.CreateStore(ctx, &aiquery.CreateStore{
    Store:         "books",
    QueryModel:    aimodel.AIModel_ALL_MINI_LM_L6_V2,
    IndexModel:    aimodel.AIModel_ALL_MINI_LM_L6_V2,
    Predicates:    []string{"author", "genre"},
    ErrorIfExists: true,
    StoreOriginal: true,
})`,
      `import (
    keyval "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
    metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    preprocess "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
)

_, err = client.Set(ctx, &aiquery.Set{
    Store: "books",
    Inputs: []*keyval.AiStoreEntry{{
        Key: &keyval.StoreInput{Value: &keyval.StoreInput_RawString{
            RawString: "A galactic empire in decline..."}},
        Value: &keyval.StoreValue{Value: map[string]*metadata.MetadataValue{
            "genre":  {Value: &metadata.MetadataValue_RawString{RawString: "SciFi"}},
            "author": {Value: &metadata.MetadataValue_RawString{RawString: "Asimov"}},
        }},
    }},
    PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
})`,
      `import (
    algorithms "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/algorithms"
)

resp, err := client.GetSimN(ctx, &aiquery.GetSimN{
    Store:            "books",
    SearchInput:      &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "space opera classics"}},
    ClosestN:         3,
    Algorithm:        algorithms.Algorithm_CosineSimilarity,
    PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing,
})

for _, entry := range resp.Entries {
    fmt.Println(entry.Key, entry.Similarity)
}`,
    ],
  },
];

export default function PythonApi(): ReactNode {
  const [sdkIdx, setSdkIdx] = useState(0);
  const [active, setActive] = useState(0);

  const sdk = SDKS[sdkIdx];
  const steps = STEP_META;
  const filename = steps[active].label.toLowerCase().replace(/\s+/g, '_');

  return (
    <section className="relative overflow-hidden bg-slate-50 py-24 dark:bg-[#1e1f20]">
      <div className="pointer-events-none absolute -right-40 top-10 h-96 w-96 rounded-full bg-primary/10 blur-3xl" />
      <div className="container relative">
        <div className="mx-auto mb-10 max-w-2xl text-center">
          <h2 className="text-4xl font-extrabold tracking-tight md:text-5xl">
            A simple, intuitive API
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-lg leading-relaxed opacity-70">
            Go from connection to semantic search in four steps, in the language
            you already work in.
          </p>
        </div>

        {/* language switcher */}
        <div className="mb-10 flex flex-col items-center gap-6">
          <div className="flex justify-center gap-1 border-b border-solid border-black/10 dark:border-white/10">
            {SDKS.map((s, idx) => {
              const isActive = idx === sdkIdx;
              return (
                <button
                  key={s.label}
                  onClick={() => setSdkIdx(idx)}
                  className={`relative -mb-px bg-transparent px-5 pb-3 pt-1 text-[0.95rem] font-semibold transition-colors duration-200 ${
                    isActive
                      ? 'text-primary'
                      : 'text-current opacity-50 hover:opacity-90'
                  }`}>
                  {s.label}
                  <span
                    className={`absolute inset-x-2 -bottom-px h-0.5 rounded-full bg-gradient-to-r from-primary to-secondary transition-transform duration-300 ${
                      isActive ? 'scale-x-100' : 'scale-x-0'
                    }`}
                  />
                </button>
              );
            })}
          </div>
          <div className="inline-flex max-w-full items-center overflow-x-auto rounded-lg border border-solid border-white/10 bg-[#0b1f28] font-mono text-sm text-slate-100 shadow-lg">
            <span className="select-none px-4 py-2.5 text-secondary">$</span>
            <code className="whitespace-nowrap bg-transparent py-2.5 pr-5">
              {sdk.install}
            </code>
          </div>
        </div>

        <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
          {/* Steps: connected timeline */}
          <ol className="m-0 flex list-none flex-col p-0">
            {steps.map((step, idx) => {
              const isActive = idx === active;
              const isDone = idx < active;
              const isLast = idx === steps.length - 1;
              return (
                <li key={step.label}>
                  <button
                    onClick={() => setActive(idx)}
                    aria-current={isActive}
                    className="flex w-full gap-4 bg-transparent text-left">
                    {/* rail + badge column (always aligned) */}
                    <div className="flex flex-none flex-col items-center self-stretch">
                      <span
                        className={`flex h-10 w-10 flex-none items-center justify-center rounded-full text-sm font-bold transition-all duration-300 ${
                          isActive
                            ? 'bg-gradient-to-br from-primary to-secondary text-white shadow-lg shadow-primary/30'
                            : isDone
                              ? 'bg-secondary/15 text-secondary'
                              : 'bg-grey-1/70 text-grey-3 dark:bg-white/10 dark:text-white/60'
                        }`}>
                        {isDone ? '✓' : idx + 1}
                      </span>
                      {!isLast && (
                        <span
                          className={`my-1 w-0.5 flex-1 rounded transition-colors duration-300 ${
                            isDone ? 'bg-secondary/60' : 'bg-grey-1/60 dark:bg-white/10'
                          }`}
                        />
                      )}
                    </div>

                    {/* content card */}
                    <div
                      className={`mb-3 flex-1 rounded-2xl border border-solid px-5 py-4 transition-all duration-300 ${
                        isActive
                          ? 'border-primary/30 bg-white shadow-xl shadow-primary/5 dark:bg-white/[0.06]'
                          : 'border-transparent hover:bg-black/[0.03] dark:hover:bg-white/[0.03]'
                      }`}>
                      <div
                        className={`text-base font-semibold transition-colors ${
                          isActive ? 'text-primary' : 'opacity-90'
                        }`}>
                        {step.label}
                      </div>
                      <div className="mt-1 text-sm leading-relaxed opacity-60">
                        {step.blurb}
                      </div>
                    </div>
                  </button>
                </li>
              );
            })}
          </ol>

          {/* Code window */}
          <div className="min-w-0 lg:sticky lg:top-24">
            <div className="flex flex-col overflow-hidden rounded-2xl border border-solid border-black/10 bg-[#282a36] shadow-2xl ring-1 ring-black/5 dark:border-white/10">
              <div className="flex items-center gap-2 border-b border-solid border-white/10 bg-white/5 px-4 py-3">
                <span className="h-3 w-3 rounded-full bg-[#ff5f56]" />
                <span className="h-3 w-3 rounded-full bg-[#ffbd2e]" />
                <span className="h-3 w-3 rounded-full bg-[#27c93f]" />
                <span className="ml-3 font-mono text-xs text-white/50">
                  {filename}.{sdk.ext}
                </span>
                <span className="ml-auto font-mono text-xs text-white/40">
                  {String(active + 1).padStart(2, '0')} /{' '}
                  {String(steps.length).padStart(2, '0')}
                </span>
              </div>
              <div
                key={`${sdkIdx}-${active}`}
                className="ahn-code-swap h-[26rem] overflow-auto text-[0.88rem] [&_pre]:!m-0 [&_pre]:!rounded-none [&_pre]:!bg-transparent [&_pre]:!p-5">
                <CodeBlock language={sdk.lang}>{sdk.code[active]}</CodeBlock>
              </div>
              {/* dot navigation inside the window footer */}
              <div className="flex items-center justify-between border-t border-solid border-white/10 bg-white/5 px-4 py-3">
                <button
                  onClick={() => setActive((i) => Math.max(0, i - 1))}
                  disabled={active === 0}
                  className="bg-transparent text-sm font-medium text-white/70 transition hover:text-white disabled:cursor-not-allowed disabled:opacity-30">
                  ← Prev
                </button>
                <div className="flex gap-1.5">
                  {steps.map((_, idx) => (
                    <button
                      key={idx}
                      aria-label={`Go to step ${idx + 1}`}
                      onClick={() => setActive(idx)}
                      className={`h-1.5 rounded-full transition-all duration-300 ${
                        idx === active
                          ? 'w-6 bg-secondary'
                          : 'w-1.5 bg-white/25 hover:bg-white/50'
                      }`}
                    />
                  ))}
                </div>
                <button
                  onClick={() =>
                    setActive((i) => Math.min(steps.length - 1, i + 1))
                  }
                  disabled={active === steps.length - 1}
                  className="bg-transparent text-sm font-medium text-secondary transition hover:text-white disabled:cursor-not-allowed disabled:opacity-30">
                  Next →
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
