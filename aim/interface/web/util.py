# Copyright 2025 Zexin Yuan
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


from typing import Optional, Self

import pydantic
from aiohttp import web
from aiohttp.typedefs import LooseHeaders

__all__ = ["RequestModel", "ResponseModel"]


class RequestModel(pydantic.BaseModel):
    @classmethod
    async def from_request(cls, request: web.Request) -> Self:
        content = await request.content.read()
        if content is None or len(content) == 0:
            content = "{}"
        return cls.model_validate_json(content)


class AuthedRequestModel(RequestModel):
    auth_token: str | None = pydantic.Field(exclude=True, default_factory=lambda: None)

    @classmethod
    async def from_request(cls, request: web.Request) -> Self:
        model = await super().from_request(request)
        model.auth_token = request.headers.get("Authorization")
        return model


class ResponseModel(pydantic.BaseModel):
    def json_response(
        self,
        *,
        status: int = 200,
        reason: Optional[str] = None,
        headers: Optional[LooseHeaders] = None,
        content_type: str = "application/json",
    ) -> web.Response:
        return web.Response(
            text=self.model_dump_json(),
            status=status,
            reason=reason,
            headers=headers,
            content_type=content_type,
        )
