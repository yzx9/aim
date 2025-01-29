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


import datetime
import os
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Generic, TypeVar

import aim.infrastructure.rdbms as rdbms
from aim.domain import Organizations, Projects, Users
from aim.util import ConfigParser, SnowflakeGenerator

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


T = TypeVar("T", bound=BaseConfig)


class BaseInterface(ABC, Generic[T]):
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
        )
        return parser, config

    def _setup_domains(self) -> None:
        # Initialize infrastructure
        repo_organization = rdbms.OrganizationRepository()
        repo_project = rdbms.ProjectRepository()
        repo_user = rdbms.UserRepository()

        # Initialize domains
        def id_generator():
            return SnowflakeGenerator(self._config.machine_id, epoch=_EPOCH)

        self._organizations = Organizations(
            repository=repo_organization, id_generator=id_generator()
        )
        self._projects = Projects(repository=repo_project, id_generator=id_generator())
        self._users = Users(repository=repo_user, id_generator=id_generator())

    def _setup_application(self) -> None:
        pass  # do nothing
