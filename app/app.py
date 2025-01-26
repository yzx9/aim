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


import os
import threading
import warnings
from dataclasses import dataclass

from aiohttp import web

import aim
from app.api import ping
from app.util import ConfigParser

__all__ = ["App"]


class App:
    """App is a singleton class that represents the AIM web application."""

    _instance_lock = threading.Lock()

    def __new__(cls, *args, **kwargs):
        if not hasattr(App, "_instance"):
            with App._instance_lock:
                if not hasattr(App, "_instance"):
                    App._instance = object.__new__(cls)

        return App._instance

    def __init__(self, root: str, **kwargs: str | int | float | bool):
        super().__init__()

        config = _load_config(root, **kwargs)
        aim.init(config.aim_config)  # This is safely since App is a singleton

        self.config = config
        self.app = _new_app(config)

    def run(self):
        web.run_app(self.app, host=self.config.host, port=self.config.port)


@dataclass
class Config:
    host: str
    port: int

    aim_config: aim.Config


def _load_config(root: str, **kwargs: str | int | float | bool) -> Config:
    parser = ConfigParser(
        cli_args=kwargs,
        env_prefix=str(kwargs["env_prefix"] or "AIM"),
        env_files=[os.path.join(root, p) for p in [".env.local", ".env"]],
    )

    return Config(
        host=parser.parse_str("APP_HOST", default="127.0.0.1"),
        port=parser.parse_int("APP_PORT", default=8080),
        aim_config=aim.Config(
            dev=parser.parse_bool("dev", default=False),
            machine_id=parser.parse_int("machine_id", default=0),
        ),
    )


def _new_app(config: Config) -> web.Application:
    app = web.Application()

    if config.aim_config.dev:
        try:
            import aiohttp_debugtoolbar

            aiohttp_debugtoolbar.setup(app)
        except ImportError:
            warnings.warn("aiohttp_debugtoolbar not installed.", ImportWarning)

    _init_routes(app)
    return app


def _init_routes(app: web.Application) -> None:
    r = app.router

    r.add_get("/api/ping", ping, name="ping")

    # added static dir
    # r.add_static("/static/", path=(PROJECT_PATH / "static"), name="static")
