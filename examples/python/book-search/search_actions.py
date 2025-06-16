import asyncio

from grpclib.client import Channel
from typing import List
from ahnlich_client_py import TRACE_HEADER
from ahnlich_client_py.grpc import keyval
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.algorithm.algorithms import Algorithm
from ahnlich_client_py.grpc.services import ai_service
from ahnlich_client_py.grpc.ai.server import GetSimNEntry


async def run_get_simn_text(
    input_query: str, span_id: str | None = None
) -> List[GetSimNEntry]:
    channel = Channel(host="127.0.0.1", port=1370)
    client = ai_service.AiServiceStub(channel)
    trace_metadata = {TRACE_HEADER: span_id} if span_id else None

    try:
        response = await client.get_sim_n(
            ai_query.GetSimN(
                store="book",
                search_input=keyval.StoreInput(raw_string=input_query),
                closest_n=5,
                algorithm=Algorithm.CosineSimilarity,
            ),
            metadata=trace_metadata,
        )
        return response.entries
    finally:
        channel.close()


async def search_phrase(span_id: str | None = None):
    input_query = input("Please enter the phrase: ")
    entries = await run_get_simn_text(input_query, span_id)

    for entry in entries:
        chapter = entry.value.value["chapter"].raw_string
        paragraph = entry.value.value["paragraph"].raw_string
        content = entry.key.raw_string
        print(f"Chapter {chapter}")
        print(f"Paragraph {paragraph}")
        print(content)
        print("\n")


loop = asyncio.get_event_loop()


def main():
    asyncio.run(search_phrase())
