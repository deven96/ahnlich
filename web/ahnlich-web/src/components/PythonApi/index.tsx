import {useEffect, useRef, useState, type ReactNode} from 'react';
import CodeBlock from '@theme/CodeBlock';

/** Brand logos for the editor filename tab, keyed by SDK label. */
const LANG_ICONS: Record<string, ReactNode> = {
  Python: (
    <svg viewBox="0 0 24 24" className="h-4 w-4 flex-none" aria-hidden>
      <path
        fill="#3776AB"
        d="M14.25.18l.9.2.73.26.59.3.45.32.34.34.25.34.16.33.1.3.04.26.02.2-.01.13V8.5l-.05.63-.13.55-.21.46-.26.38-.3.31-.33.25-.35.19-.35.14-.33.1-.3.07-.26.04-.21.02H8.77l-.69.05-.59.14-.5.22-.41.27-.33.32-.27.35-.2.36-.15.37-.1.35-.07.32-.04.27-.02.21v3.06H3.17l-.21-.03-.28-.07-.32-.12-.35-.18-.36-.26-.36-.36-.35-.46-.32-.59-.28-.73-.21-.88-.14-1.05-.05-1.23.06-1.22.16-1.04.24-.87.32-.71.36-.57.4-.44.42-.33.42-.24.4-.16.36-.1.32-.05.24-.01h.16l.06.01h8.16v-.83H6.18l-.01-2.75-.02-.37.05-.34.11-.31.17-.28.25-.26.31-.23.38-.2.44-.18.51-.15.58-.12.64-.1.71-.06.77-.04.84-.02 1.27.05zm-6.3 1.98l-.23.33-.08.41.08.41.23.34.33.22.41.09.41-.09.33-.22.23-.34.08-.41-.08-.41-.23-.33-.33-.22-.41-.09-.41.09zm13.09 3.95l.28.06.32.12.35.18.36.27.36.35.35.47.32.59.28.73.21.88.14 1.04.05 1.23-.06 1.23-.16 1.04-.24.86-.32.71-.36.57-.4.45-.42.33-.42.24-.4.16-.36.09-.32.05-.24.02-.16-.01h-8.22v.82h5.84l.01 2.76.02.36-.05.34-.11.31-.17.29-.25.25-.31.24-.38.2-.44.17-.51.15-.58.13-.64.09-.71.07-.77.04-.84.01-1.27-.04-1.07-.14-.9-.2-.73-.25-.59-.3-.45-.33-.34-.34-.25-.34-.16-.33-.1-.3-.04-.26-.02-.2.01-.13V14l.05-.64.13-.54.21-.46.26-.38.3-.31.33-.26.35-.18.35-.14.33-.1.3-.06.26-.04.21-.02.16-.01h5.98l.69-.05.59-.14.5-.21.41-.28.33-.31.27-.36.2-.36.15-.36.1-.35.07-.32.04-.28.02-.21V6.07h2.09l.14.01zm-6.47 14.25l-.23.33-.08.41.08.41.23.33.33.23.41.08.41-.08.33-.23.23-.33.08-.41-.08-.41-.23-.33-.33-.23-.41-.08-.41.08z"
      />
    </svg>
  ),
  Rust: (
    <svg viewBox="0 0 24 24" className="h-4 w-4 flex-none" aria-hidden>
      <path
        fill="#e6e6e6"
        d="M23.8346 11.7033l-1.0073-.6236a13.7268 13.7268 0 00-.0283-.2936l.8656-.8069a.3483.3483 0 00-.1154-.5783l-1.1066-.414a8.4948 8.4948 0 00-.087-.2856l.6904-.9587a.3462.3462 0 00-.2257-.5446l-1.1663-.1894a9.3574 9.3574 0 00-.1407-.2622l.49-1.0761a.3437.3437 0 00-.0274-.3372.3486.3486 0 00-.3-.1531l-1.1845.0416a6.7444 6.7444 0 00-.1873-.2268l.2723-1.153a.3472.3472 0 00-.417-.417l-1.1532.2724a14.0183 14.0183 0 00-.2273-.1873l.0415-1.1845a.3442.3442 0 00-.49-.3273l-1.076.49c-.0872-.0476-.1742-.0952-.2623-.1407l-.1893-1.1662a.3483.3483 0 00-.5447-.2258l-.9587.6904a8.4948 8.4948 0 00-.2856-.087l-.414-1.1066a.3483.3483 0 00-.5782-.1155l-.807.8657a13.7268 13.7268 0 00-.2935-.0283L12.297.1654a.3483.3483 0 00-.5942 0l-.6236 1.0073a13.7373 13.7373 0 00-.2936.0283L9.9787.31a.3483.3483 0 00-.5783.1155l-.414 1.1066a8.4948 8.4948 0 00-.2856.087L7.742.9487a.3483.3483 0 00-.5447.2258l-.1893 1.1662c-.0881.0455-.1751.0931-.2623.1407l-1.076-.49a.3442.3442 0 00-.49.3273l.0415 1.1845a14.0183 14.0183 0 00-.2273.1873L3.8672 3.425a.3472.3472 0 00-.417.417l.2724 1.153a6.7444 6.7444 0 00-.1873.2268l-1.1845-.0416a.3437.3437 0 00-.3.1531.3462.3462 0 00-.0274.3372l.49 1.0761a9.3574 9.3574 0 00-.1407.2622l-1.1662.1894a.3483.3483 0 00-.2258.5446l.6904.9587a8.4948 8.4948 0 00-.087.2856l-1.1066.414a.3483.3483 0 00-.1154.5783l.8656.8069a13.7268 13.7268 0 00-.0283.2936l-1.0073.6236a.3483.3483 0 000 .5942l1.0073.6236c.0087.0982.018.196.0283.2936l-.8656.8069a.3483.3483 0 00.1154.5783l1.1066.414c.0277.0954.0562.1904.087.2856l-.6904.9587a.3462.3462 0 00.2258.5446l1.1662.1894c.0455.0881.0931.1751.1407.2622l-.49 1.0761a.3483.3483 0 00.3274.4903l1.1845-.0416c.0615.0768.1237.153.1873.2268l-.2724 1.153a.3472.3472 0 00.417.417l1.1532-.2724c.0743.0635.1509.1252.2273.1873l-.0415 1.1845a.3442.3442 0 00.49.3273l1.076-.49c.0872.0476.1742.0952.2623.1407l.1893 1.1662a.3483.3483 0 00.5447.2258l.9587-.6904a8.4948 8.4948 0 00.2856.087l.414 1.1066a.3483.3483 0 00.5782.1155l.807-.8657c.0975.0103.1954.0196.2935.0283l.6236 1.0073a.3483.3483 0 00.5942 0l.6236-1.0073c.0982-.0087.1961-.018.2936-.0283l.807.8657a.3483.3483 0 00.5782-.1155l.414-1.1066a8.4948 8.4948 0 00.2856-.087l.9587.6904a.3462.3462 0 00.5447-.2258l.1893-1.1662c.0881-.0455.1751-.0931.2623-.1407l1.076.49a.3442.3442 0 00.49-.3273l-.0415-1.1845a14.0183 14.0183 0 00.2273-.1873l1.1532.2724a.3472.3472 0 00.417-.417l-.2724-1.153c.0635-.0743.1252-.1509.1873-.2268l1.1845.0416a.3437.3437 0 00.3-.1531.3462.3462 0 00.0274-.3372l-.49-1.0761c.0476-.0872.0952-.1742.1407-.2623l1.1662-.1893a.3483.3483 0 00.2258-.5447l-.6904-.9587c.0308-.0952.0593-.1902.087-.2856l1.1066-.414a.3483.3483 0 00.1154-.5782l-.8656-.807c.0103-.0975.0196-.1954.0283-.2935l1.0073-.6236a.3483.3483 0 000-.5942zm-6.4924 8.5931a.7166.7166 0 01-.2995-1.4014.7167.7167 0 11.2994 1.4013zm-.3426-2.3163a.6513.6513 0 00-.7738.5017l-.3589 1.6747a8.5311 8.5311 0 01-3.5646.7754 8.5311 8.5311 0 01-3.564-.7746l-.359-1.6748a.6512.6512 0 00-.7737-.5016l-1.4783.3172a8.5187 8.5187 0 01-.3821-.4506h7.1929c.0813 0 .1354-.0146.1354-.0879V14.51c0-.0732-.0541-.0878-.1354-.0878h-2.101v-1.611h2.2721c.2074 0 1.1096.0593 1.397 1.2124.0901.3542.2874 1.5075.4225 1.8765.1346.4139.6828 1.2407 1.2674 1.2407h3.5817a.7503.7503 0 00.1298-.0132 8.5628 8.5628 0 01-.408.4808l-1.5148-.3252zm-9.6437 2.2758a.7167.7167 0 11-.2996-1.4013.7167.7167 0 01.2995 1.4013zM6.3264 8.0201a.7166.7166 0 11-1.3096-.5799.7166.7166 0 011.3096.5799zm-.7024 1.6667l1.5397-.6845a.6513.6513 0 00.3306-.8607l-.317-.7167h1.2471v5.6202H5.9151a8.5468 8.5468 0 01-.276-3.2503zm4.4515-.3606V7.6634h2.9705c.1536 0 1.0836.1775 1.0836.8728 0 .5773-.7134.7842-1.2996.7842zm7.9748 2.3388c0 .2197-.0081.4375-.0241.6533h-.9021c-.0901 0-.1266.0593-.1266.1477v.4139c0 .9746-.5497 1.1867-1.0314 1.2404-.4587.0518-.9663-.1919-1.0289-.4735-.2704-1.5195-.7197-1.8446-1.4302-2.4059.8814-.5596 1.7982-1.385 1.7982-2.4896 0-1.1867-.8144-1.9348-1.3691-2.3018-.7783-.5139-1.6402-.6165-1.873-.6165H5.4772A8.5431 8.5431 0 0110.2716 3.517l.5361.5623a.6513.6513 0 00.9211.0209l.5998-.5738A8.5442 8.5442 0 0117.5195 7.643l-.4108.9276a.6515.6515 0 00.3307.8606l1.6236.7218A8.6119 8.6119 0 0119.1043 12z"
      />
    </svg>
  ),
  Node: (
    <svg viewBox="0 0 24 24" className="h-4 w-4 flex-none" aria-hidden>
      <path
        fill="#5FA04E"
        d="M12 1.85c-.27 0-.55.07-.78.2l-7.44 4.3c-.48.28-.78.8-.78 1.36v8.58c0 .56.3 1.08.78 1.36l1.95 1.12c.95.46 1.27.47 1.71.47 1.4 0 2.21-.85 2.21-2.33V8.44c0-.12-.1-.22-.22-.22h-.93c-.12 0-.22.1-.22.22v8.47c0 .66-.68 1.31-1.79.76L4.7 16.9c-.07-.04-.12-.12-.12-.2V8.12c0-.09.05-.17.12-.21l7.44-4.29c.06-.04.16-.04.22 0l7.44 4.29c.07.04.12.12.12.21v8.58c0 .08-.05.16-.12.21l-7.44 4.29c-.06.04-.16.04-.23 0l-1.9-1.13c-.06-.03-.13-.04-.18-.01-.53.3-.63.34-1.12.51-.12.04-.3.11.07.32l2.48 1.47c.24.14.5.21.78.21s.54-.07.78-.21l7.44-4.29c.48-.28.78-.8.78-1.36V7.71c0-.56-.3-1.08-.78-1.36l-7.44-4.3c-.23-.13-.5-.2-.78-.2zm1.99 6.98c-2.12 0-3.39.9-3.39 2.4 0 1.63 1.26 2.08 3.3 2.28 2.43.24 2.62.6 2.62 1.08 0 .83-.67 1.19-2.23 1.19-1.97 0-2.4-.49-2.55-1.47a.22.22 0 00-.21-.18h-.96a.22.22 0 00-.22.22c0 1.25.68 2.74 3.94 2.74 2.35 0 3.7-.93 3.7-2.55 0-1.61-1.09-2.03-3.38-2.34-2.31-.3-2.54-.46-2.54-1 0-.45.2-1.05 1.9-1.05 1.52 0 2.08.33 2.31 1.36.02.1.11.17.21.17h.96a.22.22 0 00.16-.07.22.22 0 00.06-.16c-.15-1.77-1.33-2.6-3.71-2.6z"
      />
    </svg>
  ),
  Go: (
    <svg viewBox="0 0 24 24" className="h-4 w-4 flex-none" aria-hidden>
      <path
        fill="#00ADD8"
        d="M1.811 10.231c-.047 0-.058-.023-.035-.059l.246-.315c.023-.035.081-.058.128-.058h4.172c.046 0 .058.035.035.07l-.199.303c-.023.036-.082.07-.117.07zM.047 11.306c-.047 0-.059-.024-.035-.059l.245-.315c.024-.035.082-.059.129-.059h5.328c.047 0 .07.035.058.07l-.093.28c-.012.047-.058.07-.105.07zm2.828 1.075c-.047 0-.059-.035-.035-.07l.163-.292c.023-.035.07-.07.117-.07h2.337c.047 0 .07.035.07.082l-.023.28c0 .047-.047.082-.082.082zm12.129-2.36c-.736.187-1.239.327-1.963.514-.176.046-.187.058-.34-.117-.174-.199-.303-.327-.548-.444-.737-.362-1.45-.257-2.115.175-.795.514-1.204 1.274-1.192 2.22.011.935.654 1.706 1.577 1.835.795.105 1.462-.175 1.988-.77.105-.13.198-.27.315-.434h-2.242c-.245 0-.304-.152-.222-.35.152-.362.432-.97.596-1.274a.315.315 0 01.292-.187h4.253c-.023.316-.023.631-.07.947a4.983 4.983 0 01-.958 2.29c-.841 1.11-1.94 1.8-3.33 1.986-1.145.152-2.209-.07-3.143-.77-.865-.655-1.356-1.52-1.484-2.595-.152-1.274.222-2.419.993-3.424.83-1.086 1.928-1.776 3.272-2.02 1.098-.2 2.15-.07 3.096.571.62.41 1.063.97 1.356 1.648.07.105.023.164-.117.2m3.868 6.461c-1.064-.024-2.034-.328-2.852-1.029a3.665 3.665 0 01-1.262-2.255c-.21-1.32.152-2.489.947-3.529.853-1.122 1.881-1.706 3.272-1.95 1.192-.21 2.314-.095 3.33.595.923.63 1.496 1.484 1.648 2.605.198 1.578-.257 2.863-1.34 3.962-.771.783-1.718 1.273-2.805 1.495-.315.06-.63.07-.788.07zm2.78-4.72c-.011-.153-.011-.27-.034-.387-.21-1.157-1.274-1.81-2.384-1.554-1.087.245-1.788.935-2.045 2.033-.21.912.234 1.835 1.075 2.21.643.28 1.285.244 1.905-.07.923-.48 1.425-1.228 1.484-2.222z"
      />
    </svg>
  ),
};

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

/** vh of scroll distance allotted to each step transition while pinned */
const SCROLL_PER_STEP = 50;

export default function PythonApi(): ReactNode {
  const [sdkIdx, setSdkIdx] = useState(0);
  const [active, setActive] = useState(0);
  const [pinned, setPinned] = useState(false);
  const [progress, setProgress] = useState(0);
  const [installCopied, setInstallCopied] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);

  const sdk = SDKS[sdkIdx];
  const steps = STEP_META;
  const filename = steps[active].label.toLowerCase().replace(/\s+/g, '_');

  const copyInstall = () => {
    navigator.clipboard?.writeText(sdk.install).then(() => {
      setInstallCopied(true);
      setTimeout(() => setInstallCopied(false), 1600);
    });
  };

  // Enable the pinned scroll experience on large screens only; smaller screens
  // fall back to a normal, click-driven stacked layout.
  useEffect(() => {
    const mq = window.matchMedia('(min-width: 1024px)');
    const update = () => setPinned(mq.matches);
    update();
    mq.addEventListener('change', update);
    return () => mq.removeEventListener('change', update);
  }, []);

  // Map scroll position within the pinned section to the active step.
  useEffect(() => {
    if (!pinned) return;
    let raf = 0;
    const onScroll = () => {
      if (raf) return;
      raf = requestAnimationFrame(() => {
        raf = 0;
        const el = sectionRef.current;
        if (!el) return;
        const rect = el.getBoundingClientRect();
        const total = el.offsetHeight - window.innerHeight;
        if (total <= 0) return;
        const scrolled = Math.min(Math.max(-rect.top, 0), total);
        const p = scrolled / total;
        setProgress(p);
        setActive(
          Math.min(steps.length - 1, Math.max(0, Math.round(p * (steps.length - 1)))),
        );
      });
    };
    window.addEventListener('scroll', onScroll, {passive: true});
    onScroll();
    return () => {
      window.removeEventListener('scroll', onScroll);
      if (raf) cancelAnimationFrame(raf);
    };
  }, [pinned, steps.length]);

  // Clicking a step scrolls the page so the pin lands on it (or just sets it
  // directly when not pinned).
  const goToStep = (idx: number) => {
    const el = sectionRef.current;
    if (!pinned || !el) {
      setActive(idx);
      return;
    }
    const rect = el.getBoundingClientRect();
    const absTop = window.scrollY + rect.top;
    const total = el.offsetHeight - window.innerHeight;
    const p = steps.length > 1 ? idx / (steps.length - 1) : 0;
    window.scrollTo({top: absTop + p * total, behavior: 'smooth'});
  };

  return (
    <section
      ref={sectionRef}
      style={
        pinned
          ? {height: `calc(100vh + ${(steps.length - 1) * SCROLL_PER_STEP}vh)`}
          : undefined
      }
      className="relative bg-gradient-to-b from-white via-[#eaf1fb] to-white dark:from-[#08161d] dark:via-[#0c2230] dark:to-[#08161d]">
      <div
        className={
          pinned
            ? 'sticky top-0 flex min-h-screen items-center overflow-hidden pb-6 pt-20'
            : 'relative overflow-hidden py-24'
        }>
        {/* ambient backdrop */}
        <div className="ahn-grid pointer-events-none absolute inset-0 opacity-40 [mask-image:radial-gradient(ellipse_at_center,black,transparent_78%)] dark:opacity-20" />
        <div className="pointer-events-none absolute -right-40 top-10 h-96 w-96 rounded-full bg-primary/10 blur-3xl" />
        <div className="pointer-events-none absolute -left-40 bottom-10 h-96 w-96 rounded-full bg-secondary/10 blur-3xl" />

        <div className="container relative w-full">
          <div className="mx-auto mb-8 max-w-2xl text-center">
            <h2 className="text-3xl font-extrabold tracking-tight text-[#0c1e28] dark:text-white md:text-5xl">
              A simple, intuitive API
            </h2>
            <p className="mx-auto mt-3 max-w-xl text-lg leading-relaxed text-[#5a6b86] dark:text-slate-300/80">
              Go from connection to semantic search in four steps, in the
              language you already work in.
            </p>
          </div>

          {/* language switcher */}
          <div className="mb-8 flex flex-col items-center gap-5">
            <div className="flex justify-center gap-1 border-b border-solid border-black/10 dark:border-white/10">
              {SDKS.map((s, idx) => {
                const isActive = idx === sdkIdx;
                return (
                  <button
                    key={s.label}
                    onClick={() => setSdkIdx(idx)}
                    className={`relative -mb-px flex items-center gap-1.5 bg-transparent px-5 pb-3 pt-1 text-[0.95rem] font-semibold transition-all duration-200 ${
                      isActive
                        ? 'text-primary'
                        : 'text-current opacity-50 grayscale hover:opacity-90 hover:grayscale-0'
                    }`}>
                    <span className="[&_svg]:h-4 [&_svg]:w-4">
                      {LANG_ICONS[s.label]}
                    </span>
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
            {/* install command — mini terminal with copy */}
            <div className="group flex max-w-full items-center gap-3 overflow-hidden rounded-lg border border-solid border-white/10 bg-[#0b1f28] py-1.5 pl-4 pr-2 font-mono text-sm text-slate-100 shadow-lg">
              <span className="select-none text-secondary">$</span>
              <code className="overflow-x-auto whitespace-nowrap border-0 bg-transparent p-0 py-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                {sdk.install}
              </code>
              <button
                onClick={copyInstall}
                aria-label={installCopied ? 'Copied' : 'Copy install command'}
                title={installCopied ? 'Copied' : 'Copy'}
                className="flex h-8 w-8 flex-none items-center justify-center rounded-md bg-transparent text-white/50 transition-colors hover:bg-white/10 hover:text-white">
                {installCopied ? (
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4 text-secondary">
                    <path d="M20 6 9 17l-5-5" />
                  </svg>
                ) : (
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4">
                    <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                  </svg>
                )}
              </button>
            </div>
          </div>

          <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
            {/* Steps: connected timeline with a scroll-progress rail */}
            <ol className="relative m-0 flex list-none flex-col p-0">
              {steps.map((step, idx) => {
                const isActive = idx === active;
                const isDone = idx < active;
                const isLast = idx === steps.length - 1;
                return (
                  <li key={step.label}>
                    <button
                      onClick={() => goToStep(idx)}
                      aria-current={isActive}
                      className="flex w-full gap-4 bg-transparent text-left">
                      {/* rail + badge column (always aligned) */}
                      <div className="flex flex-none flex-col items-center self-stretch">
                        <span
                          className={`flex h-10 w-10 flex-none items-center justify-center rounded-full text-sm font-bold transition-all duration-300 ${
                            isActive
                              ? 'scale-110 bg-gradient-to-br from-primary to-secondary text-white shadow-lg shadow-primary/30'
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

            {/* Editor — a single clean frame, no nested window chrome */}
            <div className="min-w-0">
              <div className="overflow-hidden rounded-xl bg-[#0b1f28] shadow-[0_24px_60px_-24px_rgba(8,22,29,0.6)] ring-1 ring-black/5 dark:ring-white/10">
                {/* tab strip */}
                <div className="flex items-end gap-px border-b border-solid border-white/[0.07] bg-[#081820] px-2 pt-2">
                  <div className="flex items-center gap-2 rounded-t-lg bg-[#0b1f28] px-3.5 py-2 font-mono text-xs text-white/85">
                    {LANG_ICONS[sdk.label]}
                    {filename}.{sdk.ext}
                  </div>
                  <span className="ml-auto select-none pb-2 pr-2 font-mono text-[11px] tabular-nums text-white/35">
                    {String(active + 1).padStart(2, '0')} /{' '}
                    {String(steps.length).padStart(2, '0')}
                  </span>
                </div>
                {/* thin scroll-progress bar */}
                <div className="h-[3px] w-full bg-white/[0.06]">
                  <div
                    className="h-full bg-gradient-to-r from-primary to-secondary transition-[width] duration-150"
                    style={{
                      width: `${
                        pinned
                          ? progress * 100
                          : (active / Math.max(1, steps.length - 1)) * 100
                      }%`,
                    }}
                  />
                </div>
                {/* code */}
                <div
                  key={`${sdkIdx}-${active}`}
                  className="ahn-api-editor ahn-code-swap h-[24rem] overflow-auto xl:h-[26rem]">
                  <CodeBlock language={sdk.lang}>{sdk.code[active]}</CodeBlock>
                </div>
                {/* step navigation */}
                <div className="flex items-center justify-between border-t border-solid border-white/[0.07] bg-[#081820] px-3 py-2.5">
                  <button
                    onClick={() => goToStep(Math.max(0, active - 1))}
                    disabled={active === 0}
                    className="inline-flex items-center gap-1 rounded-md bg-transparent px-2 py-1 text-sm font-medium text-white/60 transition hover:bg-white/5 hover:text-white disabled:cursor-not-allowed disabled:opacity-25 disabled:hover:bg-transparent">
                    ← Prev
                  </button>
                  <div className="flex items-center gap-1.5">
                    {steps.map((step, idx) => (
                      <button
                        key={idx}
                        aria-label={`Go to step ${idx + 1}: ${step.label}`}
                        title={step.label}
                        onClick={() => goToStep(idx)}
                        className={`h-1.5 rounded-full transition-all duration-300 ${
                          idx === active
                            ? 'w-6 bg-gradient-to-r from-primary to-secondary'
                            : 'w-1.5 bg-white/20 hover:bg-white/40'
                        }`}
                      />
                    ))}
                  </div>
                  <button
                    onClick={() => goToStep(Math.min(steps.length - 1, active + 1))}
                    disabled={active === steps.length - 1}
                    className="inline-flex items-center gap-1 rounded-md bg-transparent px-2 py-1 text-sm font-medium text-secondary transition hover:bg-white/5 hover:text-white disabled:cursor-not-allowed disabled:opacity-25 disabled:hover:bg-transparent">
                    Next →
                  </button>
                </div>
              </div>
              {pinned && (
                <p className="mt-4 text-center text-xs font-medium uppercase tracking-wider text-[#8299a3] dark:text-white/30">
                  Scroll to walk through each step
                </p>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
