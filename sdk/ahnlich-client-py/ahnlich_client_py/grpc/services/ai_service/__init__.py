# Generated by the protocol buffer compiler.  DO NOT EDIT!
# sources: services/ai_service.proto
# plugin: python-betterproto
# This file has been @generated

from dataclasses import dataclass
from typing import TYPE_CHECKING, Dict, Optional

import betterproto
import grpclib
from betterproto.grpc.grpclib_server import ServiceBase

from ...ai import pipeline as __ai_pipeline__
from ...ai import query as __ai_query__
from ...ai import server as __ai_server__

if TYPE_CHECKING:
    import grpclib.server
    from betterproto.grpc.grpclib_client import MetadataLike
    from grpclib.metadata import Deadline


class AiServiceStub(betterproto.ServiceStub):
    async def create_store(
        self,
        ai_query_create_store: "__ai_query__.CreateStore",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Unit":
        return await self._unary_unary(
            "/services.ai_service.AIService/CreateStore",
            ai_query_create_store,
            __ai_server__.Unit,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def get_key(
        self,
        ai_query_get_key: "__ai_query__.GetKey",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Get":
        return await self._unary_unary(
            "/services.ai_service.AIService/GetKey",
            ai_query_get_key,
            __ai_server__.Get,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def get_pred(
        self,
        ai_query_get_pred: "__ai_query__.GetPred",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Get":
        return await self._unary_unary(
            "/services.ai_service.AIService/GetPred",
            ai_query_get_pred,
            __ai_server__.Get,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def get_sim_n(
        self,
        ai_query_get_sim_n: "__ai_query__.GetSimN",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.GetSimN":
        return await self._unary_unary(
            "/services.ai_service.AIService/GetSimN",
            ai_query_get_sim_n,
            __ai_server__.GetSimN,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def ping(
        self,
        ai_query_ping: "__ai_query__.Ping",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Pong":
        return await self._unary_unary(
            "/services.ai_service.AIService/Ping",
            ai_query_ping,
            __ai_server__.Pong,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def create_pred_index(
        self,
        ai_query_create_pred_index: "__ai_query__.CreatePredIndex",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.CreateIndex":
        return await self._unary_unary(
            "/services.ai_service.AIService/CreatePredIndex",
            ai_query_create_pred_index,
            __ai_server__.CreateIndex,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def create_non_linear_algorithm_index(
        self,
        ai_query_create_non_linear_algorithm_index: "__ai_query__.CreateNonLinearAlgorithmIndex",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.CreateIndex":
        return await self._unary_unary(
            "/services.ai_service.AIService/CreateNonLinearAlgorithmIndex",
            ai_query_create_non_linear_algorithm_index,
            __ai_server__.CreateIndex,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def drop_pred_index(
        self,
        ai_query_drop_pred_index: "__ai_query__.DropPredIndex",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Del":
        return await self._unary_unary(
            "/services.ai_service.AIService/DropPredIndex",
            ai_query_drop_pred_index,
            __ai_server__.Del,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def drop_non_linear_algorithm_index(
        self,
        ai_query_drop_non_linear_algorithm_index: "__ai_query__.DropNonLinearAlgorithmIndex",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Del":
        return await self._unary_unary(
            "/services.ai_service.AIService/DropNonLinearAlgorithmIndex",
            ai_query_drop_non_linear_algorithm_index,
            __ai_server__.Del,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def del_key(
        self,
        ai_query_del_key: "__ai_query__.DelKey",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Del":
        return await self._unary_unary(
            "/services.ai_service.AIService/DelKey",
            ai_query_del_key,
            __ai_server__.Del,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def drop_store(
        self,
        ai_query_drop_store: "__ai_query__.DropStore",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Del":
        return await self._unary_unary(
            "/services.ai_service.AIService/DropStore",
            ai_query_drop_store,
            __ai_server__.Del,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def list_clients(
        self,
        ai_query_list_clients: "__ai_query__.ListClients",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.ClientList":
        return await self._unary_unary(
            "/services.ai_service.AIService/ListClients",
            ai_query_list_clients,
            __ai_server__.ClientList,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def list_stores(
        self,
        ai_query_list_stores: "__ai_query__.ListStores",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.StoreList":
        return await self._unary_unary(
            "/services.ai_service.AIService/ListStores",
            ai_query_list_stores,
            __ai_server__.StoreList,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def purge_stores(
        self,
        ai_query_purge_stores: "__ai_query__.PurgeStores",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Del":
        return await self._unary_unary(
            "/services.ai_service.AIService/PurgeStores",
            ai_query_purge_stores,
            __ai_server__.Del,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def set(
        self,
        ai_query_set: "__ai_query__.Set",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_server__.Set":
        return await self._unary_unary(
            "/services.ai_service.AIService/Set",
            ai_query_set,
            __ai_server__.Set,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )

    async def pipeline(
        self,
        ai_pipeline_ai_request_pipeline: "__ai_pipeline__.AiRequestPipeline",
        *,
        timeout: Optional[float] = None,
        deadline: Optional["Deadline"] = None,
        metadata: Optional["MetadataLike"] = None
    ) -> "__ai_pipeline__.AiResponsePipeline":
        return await self._unary_unary(
            "/services.ai_service.AIService/Pipeline",
            ai_pipeline_ai_request_pipeline,
            __ai_pipeline__.AiResponsePipeline,
            timeout=timeout,
            deadline=deadline,
            metadata=metadata,
        )


class AiServiceBase(ServiceBase):
    async def create_store(
        self, ai_query_create_store: "__ai_query__.CreateStore"
    ) -> "__ai_server__.Unit":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def get_key(
        self, ai_query_get_key: "__ai_query__.GetKey"
    ) -> "__ai_server__.Get":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def get_pred(
        self, ai_query_get_pred: "__ai_query__.GetPred"
    ) -> "__ai_server__.Get":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def get_sim_n(
        self, ai_query_get_sim_n: "__ai_query__.GetSimN"
    ) -> "__ai_server__.GetSimN":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def ping(self, ai_query_ping: "__ai_query__.Ping") -> "__ai_server__.Pong":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def create_pred_index(
        self, ai_query_create_pred_index: "__ai_query__.CreatePredIndex"
    ) -> "__ai_server__.CreateIndex":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def create_non_linear_algorithm_index(
        self,
        ai_query_create_non_linear_algorithm_index: "__ai_query__.CreateNonLinearAlgorithmIndex",
    ) -> "__ai_server__.CreateIndex":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def drop_pred_index(
        self, ai_query_drop_pred_index: "__ai_query__.DropPredIndex"
    ) -> "__ai_server__.Del":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def drop_non_linear_algorithm_index(
        self,
        ai_query_drop_non_linear_algorithm_index: "__ai_query__.DropNonLinearAlgorithmIndex",
    ) -> "__ai_server__.Del":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def del_key(
        self, ai_query_del_key: "__ai_query__.DelKey"
    ) -> "__ai_server__.Del":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def drop_store(
        self, ai_query_drop_store: "__ai_query__.DropStore"
    ) -> "__ai_server__.Del":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def list_clients(
        self, ai_query_list_clients: "__ai_query__.ListClients"
    ) -> "__ai_server__.ClientList":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def list_stores(
        self, ai_query_list_stores: "__ai_query__.ListStores"
    ) -> "__ai_server__.StoreList":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def purge_stores(
        self, ai_query_purge_stores: "__ai_query__.PurgeStores"
    ) -> "__ai_server__.Del":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def set(self, ai_query_set: "__ai_query__.Set") -> "__ai_server__.Set":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def pipeline(
        self, ai_pipeline_ai_request_pipeline: "__ai_pipeline__.AiRequestPipeline"
    ) -> "__ai_pipeline__.AiResponsePipeline":
        raise grpclib.GRPCError(grpclib.const.Status.UNIMPLEMENTED)

    async def __rpc_create_store(
        self,
        stream: "grpclib.server.Stream[__ai_query__.CreateStore, __ai_server__.Unit]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.create_store(request)
        await stream.send_message(response)

    async def __rpc_get_key(
        self, stream: "grpclib.server.Stream[__ai_query__.GetKey, __ai_server__.Get]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.get_key(request)
        await stream.send_message(response)

    async def __rpc_get_pred(
        self, stream: "grpclib.server.Stream[__ai_query__.GetPred, __ai_server__.Get]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.get_pred(request)
        await stream.send_message(response)

    async def __rpc_get_sim_n(
        self,
        stream: "grpclib.server.Stream[__ai_query__.GetSimN, __ai_server__.GetSimN]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.get_sim_n(request)
        await stream.send_message(response)

    async def __rpc_ping(
        self, stream: "grpclib.server.Stream[__ai_query__.Ping, __ai_server__.Pong]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.ping(request)
        await stream.send_message(response)

    async def __rpc_create_pred_index(
        self,
        stream: "grpclib.server.Stream[__ai_query__.CreatePredIndex, __ai_server__.CreateIndex]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.create_pred_index(request)
        await stream.send_message(response)

    async def __rpc_create_non_linear_algorithm_index(
        self,
        stream: "grpclib.server.Stream[__ai_query__.CreateNonLinearAlgorithmIndex, __ai_server__.CreateIndex]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.create_non_linear_algorithm_index(request)
        await stream.send_message(response)

    async def __rpc_drop_pred_index(
        self,
        stream: "grpclib.server.Stream[__ai_query__.DropPredIndex, __ai_server__.Del]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.drop_pred_index(request)
        await stream.send_message(response)

    async def __rpc_drop_non_linear_algorithm_index(
        self,
        stream: "grpclib.server.Stream[__ai_query__.DropNonLinearAlgorithmIndex, __ai_server__.Del]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.drop_non_linear_algorithm_index(request)
        await stream.send_message(response)

    async def __rpc_del_key(
        self, stream: "grpclib.server.Stream[__ai_query__.DelKey, __ai_server__.Del]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.del_key(request)
        await stream.send_message(response)

    async def __rpc_drop_store(
        self, stream: "grpclib.server.Stream[__ai_query__.DropStore, __ai_server__.Del]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.drop_store(request)
        await stream.send_message(response)

    async def __rpc_list_clients(
        self,
        stream: "grpclib.server.Stream[__ai_query__.ListClients, __ai_server__.ClientList]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.list_clients(request)
        await stream.send_message(response)

    async def __rpc_list_stores(
        self,
        stream: "grpclib.server.Stream[__ai_query__.ListStores, __ai_server__.StoreList]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.list_stores(request)
        await stream.send_message(response)

    async def __rpc_purge_stores(
        self,
        stream: "grpclib.server.Stream[__ai_query__.PurgeStores, __ai_server__.Del]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.purge_stores(request)
        await stream.send_message(response)

    async def __rpc_set(
        self, stream: "grpclib.server.Stream[__ai_query__.Set, __ai_server__.Set]"
    ) -> None:
        request = await stream.recv_message()
        response = await self.set(request)
        await stream.send_message(response)

    async def __rpc_pipeline(
        self,
        stream: "grpclib.server.Stream[__ai_pipeline__.AiRequestPipeline, __ai_pipeline__.AiResponsePipeline]",
    ) -> None:
        request = await stream.recv_message()
        response = await self.pipeline(request)
        await stream.send_message(response)

    def __mapping__(self) -> Dict[str, grpclib.const.Handler]:
        return {
            "/services.ai_service.AIService/CreateStore": grpclib.const.Handler(
                self.__rpc_create_store,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.CreateStore,
                __ai_server__.Unit,
            ),
            "/services.ai_service.AIService/GetKey": grpclib.const.Handler(
                self.__rpc_get_key,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.GetKey,
                __ai_server__.Get,
            ),
            "/services.ai_service.AIService/GetPred": grpclib.const.Handler(
                self.__rpc_get_pred,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.GetPred,
                __ai_server__.Get,
            ),
            "/services.ai_service.AIService/GetSimN": grpclib.const.Handler(
                self.__rpc_get_sim_n,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.GetSimN,
                __ai_server__.GetSimN,
            ),
            "/services.ai_service.AIService/Ping": grpclib.const.Handler(
                self.__rpc_ping,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.Ping,
                __ai_server__.Pong,
            ),
            "/services.ai_service.AIService/CreatePredIndex": grpclib.const.Handler(
                self.__rpc_create_pred_index,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.CreatePredIndex,
                __ai_server__.CreateIndex,
            ),
            "/services.ai_service.AIService/CreateNonLinearAlgorithmIndex": grpclib.const.Handler(
                self.__rpc_create_non_linear_algorithm_index,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.CreateNonLinearAlgorithmIndex,
                __ai_server__.CreateIndex,
            ),
            "/services.ai_service.AIService/DropPredIndex": grpclib.const.Handler(
                self.__rpc_drop_pred_index,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.DropPredIndex,
                __ai_server__.Del,
            ),
            "/services.ai_service.AIService/DropNonLinearAlgorithmIndex": grpclib.const.Handler(
                self.__rpc_drop_non_linear_algorithm_index,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.DropNonLinearAlgorithmIndex,
                __ai_server__.Del,
            ),
            "/services.ai_service.AIService/DelKey": grpclib.const.Handler(
                self.__rpc_del_key,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.DelKey,
                __ai_server__.Del,
            ),
            "/services.ai_service.AIService/DropStore": grpclib.const.Handler(
                self.__rpc_drop_store,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.DropStore,
                __ai_server__.Del,
            ),
            "/services.ai_service.AIService/ListClients": grpclib.const.Handler(
                self.__rpc_list_clients,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.ListClients,
                __ai_server__.ClientList,
            ),
            "/services.ai_service.AIService/ListStores": grpclib.const.Handler(
                self.__rpc_list_stores,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.ListStores,
                __ai_server__.StoreList,
            ),
            "/services.ai_service.AIService/PurgeStores": grpclib.const.Handler(
                self.__rpc_purge_stores,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.PurgeStores,
                __ai_server__.Del,
            ),
            "/services.ai_service.AIService/Set": grpclib.const.Handler(
                self.__rpc_set,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_query__.Set,
                __ai_server__.Set,
            ),
            "/services.ai_service.AIService/Pipeline": grpclib.const.Handler(
                self.__rpc_pipeline,
                grpclib.const.Cardinality.UNARY_UNARY,
                __ai_pipeline__.AiRequestPipeline,
                __ai_pipeline__.AiResponsePipeline,
            ),
        }
