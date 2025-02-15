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

# pyright: strict

import datetime
import os
from abc import ABC, abstractmethod
from dataclasses import dataclass

from sqlalchemy.ext.asyncio import create_async_engine

import aim.infrastructure.rdbms as rdbms
from aim.application.application import Application
from aim.domain import Organizations, Projects, Users
from aim.util import ConfigParser, SnowflakeGenerator, SQLAlchemyAsyncSessionHandler

__all__ = ["BaseConfig", "BaseInterface"]

# Epoch for Snowflake IDs, default: 2025-01-01 00:00:00 UTC in milliseconds
_EPOCH = int(
    (datetime.datetime(2025, 1, 1) - datetime.datetime(1970, 1, 1)).total_seconds()
    * 1000
)


@dataclass
class BaseConfig:
    dev: bool
    machine_id: int
    cert_sign: str

    rdbms_type: str
    rdbms_connect_string: str


class _Repository[T: BaseConfig]:
    def __init__(self, config: T) -> None:
        super().__init__()

        match config.rdbms_type:
            case "sqlite" if config.rdbms_connect_string == ":memory:":
                # Use a static pool to enable access a memory database in multiple threads
                from sqlalchemy.pool import StaticPool

                engine = create_async_engine(
                    "sqlite+aiosqlite:///:memory:",
                    connect_args={"check_same_thread": False},
                    poolclass=StaticPool,
                )

            case "sqlite":
                url = f"sqlite+aiosqlite:///{config.rdbms_connect_string}"
                engine = create_async_engine(url)

            case "postgresql":
                url = f"postgresql+asyncpg://{config.rdbms_connect_string}"
                engine = create_async_engine(url)

            case _:
                raise ValueError(f"Unsupported RDBMS type: {config.rdbms_type}")

        session_manager = SQLAlchemyAsyncSessionHandler(engine)

        self.organizations = rdbms.OrganizationRepository(session_manager)
        self.projects = rdbms.ProjectRepository(session_manager)
        self.project_items = rdbms.ProjectItemRepository(session_manager)
        self.users = rdbms.UserRepository(session_manager)


class BaseInterface[T: BaseConfig](ABC):
    _organizations: Organizations
    _projects: Projects
    _users: Users

    _config: T

    def __init__(self, root: str, **kwargs: str | int | float | bool):
        super().__init__()
        self._config = self._load_config(root, **kwargs)
        self._setup_domains()
        self._setup_application()

    @abstractmethod
    def _load_config(self, root: str, **kwargs: str | int | float | bool) -> T: ...

    def _load_base_config(
        self, root: str, **kwargs: str | int | float | bool
    ) -> tuple[ConfigParser, BaseConfig]:
        parser = ConfigParser(
            cli_args=kwargs,
            env_prefix=str(kwargs["env_prefix"] or "AIM"),
            env_files=[os.path.join(root, p) for p in [".env.local", ".env"]],
        )
        config = BaseConfig(
            dev=parser.parse_bool("DEV", default=False),
            machine_id=parser.parse_int("MACHINE_ID", default=0),
            cert_sign=parser.parse_str("CERT_SIGN", default=""),
            rdbms_type=parser.parse_str("RDBMS_TYPE", default="sqlite"),
            rdbms_connect_string=parser.parse_str(
                "RDBMS_CONNECT_STRING", default=":memory:"
            ),
        )
        return parser, config

    def _setup_domains(self) -> None:
        # Initialize infrastructure
        repository = _Repository(self._config)

        # Initialize utils
        def id_generator():
            return SnowflakeGenerator(self._config.machine_id, epoch=_EPOCH)

        # Initialize domains
        self._organizations = Organizations(
            repository=repository, id_generator=id_generator()
        )
        self._projects = Projects(repository=repository, id_generator=id_generator())
        self._users = Users(
            repository=repository,
            id_generator=id_generator(),
            cert_sign=self._config.cert_sign,
        )

    def _setup_application(self) -> None:
        self._application = Application(users=self._users)
