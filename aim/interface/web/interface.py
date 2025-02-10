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


import dataclasses
import warnings

from aiohttp import web

from aim.interface._interface import BaseConfig, BaseInterface
from aim.interface.web.handler_organization import OrganizationsHandler
from aim.interface.web.handler_session import SessionHandler

__all__ = ["WebInterface"]


@dataclasses.dataclass
class WebConfig(BaseConfig):
    host: str
    port: int


class WebInterface(BaseInterface):
    def run(self) -> None:
        web.run_app(self._app, host=self._config.host, port=self._config.port)

    def _load_config(self, root: str, **kwargs: str | int | float | bool) -> WebConfig:
        parser, base_config = self._load_base_config(root, **kwargs)

        return WebConfig(
            **dataclasses.asdict(base_config),
            host=parser.parse_str("APP_HOST", default="127.0.0.1"),
            port=parser.parse_int("APP_PORT", default=8080),
        )

    def _setup_app(self) -> None:
        self._app = web.Application()

        # Store the Users domain service in the application
        self._app["users"] = self._users

        if self._config.dev:
            try:
                import aiohttp_debugtoolbar

                aiohttp_debugtoolbar.setup(self._app)
            except ImportError:
                warnings.warn("aiohttp_debugtoolbar not installed.", ImportWarning)

        self._add_routes()

    def _add_routes(self) -> None:
        r = self._app.router

        # added static dir
        # r.add_static("/static/", path=(PROJECT_PATH / "static"), name="static")

        r.add_get("/api/ping", _ping, name="get_ping")
        r.add_post("/api/ping", _ping, name="post_ping")

        h = SessionHandler(self._application)
        r.add_post("/api/sessions", h._post, name="post_sessions")

        h = OrganizationsHandler(self._organizations)
        r.add_post("/api/organizations", h._post, name="post_organizations")
        r.add_get("/api/organizations/{id}", h._get, name="get_organization")

    def _get_token(self, request: web.Request) -> str | None:
        return request.headers.get("Authorization")


async def _ping(request: web.Request) -> web.Response:
    return web.json_response("pong")
